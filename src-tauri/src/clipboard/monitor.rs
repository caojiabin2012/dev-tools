use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use sha2::{Sha256, Digest};
use crate::clipboard::database::Database;
use crate::clipboard::{schedule_clipboard_read, CLIPBOARD_WRITING, LAST_CLIPBOARD_HASH, SKIP_NEXT_IMAGE};
use crate::clipboard::image_io::gif_dimensions;
use crate::clipboard::file_io::{
    paths_to_storage_json, build_file_meta, hash_file_paths,
};

#[cfg(not(target_os = "windows"))]
use crate::clipboard::clipboard_io_lock;
#[cfg(not(target_os = "windows"))]
use crate::clipboard::image_io::{
    read_gif_from_path, is_gif_path, is_static_image_path, read_image_file_from_path,
    try_read_gif_from_clipboard_formats, try_read_gif_from_hdrop, clipboard_has_gif_file,
};
#[cfg(not(target_os = "windows"))]
use crate::clipboard::file_io::{get_clipboard_file_paths, should_save_as_file_record};

#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, WM_CLIPBOARDUPDATE, WNDCLASSW, MSG, GWLP_USERDATA, SetWindowLongPtrW,
    GetWindowLongPtrW, HWND_MESSAGE,
};

pub struct ClipboardMonitor {
    db: Arc<Mutex<Database>>,
    running: Arc<Mutex<bool>>,
}

impl ClipboardMonitor {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let db = self.db.clone();
        let running = self.running.clone();

        #[cfg(target_os = "windows")]
        {
            self.start_windows_listener(db, running);
        }

        #[cfg(not(target_os = "windows"))]
        {
            crate::clipboard::listener::start(running);
        }
    }

    #[cfg(target_os = "windows")]
    fn start_windows_listener(&self, db: Arc<Mutex<Database>>, running: Arc<Mutex<bool>>) {
        std::thread::spawn(move || {
            loop {
                if !*running.lock().unwrap() {
                    break;
                }

                let exited = run_windows_listener_once(&db, &running);
                if !*running.lock().unwrap() {
                    break;
                }
                if exited {
                    log::warn!("Clipboard listener message loop exited, restarting in 1s...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
            log::info!("Clipboard Windows listener stopped.");
        });
    }
}

#[cfg(target_os = "windows")]
fn run_windows_listener_once(_db: &Arc<Mutex<Database>>, running: &Arc<Mutex<bool>>) -> bool {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let window_class = "DevToolsClipboardListener";
        let window_class_w: Vec<u16> = window_class.encode_utf16().chain(std::iter::once(0)).collect();

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(window_class_w.as_ptr()),
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            use windows::Win32::Foundation::GetLastError;
            // 1410 = ERROR_CLASS_ALREADY_EXISTS，listener 重启时可忽略
            if GetLastError().0 != 1410 {
                log::error!(
                    "Failed to register clipboard listener window class: {:?}",
                    GetLastError()
                );
                return false;
            }
        }

        let hwnd = match CreateWindowExW(
            Default::default(),
            PCWSTR(window_class_w.as_ptr()),
            PCWSTR(std::ptr::null()),
            Default::default(),
            0, 0, 0, 0,
            Some(HWND_MESSAGE),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        ) {
            Ok(hwnd) => hwnd,
            Err(e) => {
                log::error!("Failed to create clipboard listener window: {:?}", e);
                return false;
            }
        };

        let shared = Box::new(ClipboardShared {
            running: running.clone(),
        });
        let ptr = Box::into_raw(shared);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

        if let Err(e) = AddClipboardFormatListener(hwnd) {
            log::error!("Failed to add clipboard listener: {:?}", e);
            let _ = Box::from_raw(ptr);
            return false;
        }

        log::info!("Clipboard event-driven listener started.");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !*running.lock().unwrap() {
                PostQuitMessage(0);
                break;
            }
            DispatchMessageW(&msg);
        }

        let _ = RemoveClipboardFormatListener(hwnd);
        let _ = DestroyWindow(hwnd);
        let _ = Box::from_raw(ptr);
        true
    }
}

struct ClipboardShared {
    running: Arc<Mutex<bool>>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            if SKIP_NEXT_IMAGE.load(Ordering::SeqCst) || CLIPBOARD_WRITING.load(Ordering::SeqCst) {
                return LRESULT(0);
            }
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ptr != 0 {
                let shared = &*(ptr as *const ClipboardShared);
                if *shared.running.lock().unwrap() {
                    schedule_clipboard_read();
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn save_gif_if_changed(
    db: &Arc<Mutex<Database>>,
    gif_data: Vec<u8>,
) -> Result<(), String> {
    if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
        *LAST_CLIPBOARD_HASH.lock().unwrap() = hash_bytes(&gif_data);
        return Ok(());
    }

    let hash = hash_bytes(&gif_data);
    let hash_changed = {
        let h = LAST_CLIPBOARD_HASH.lock().unwrap();
        *h != hash
    };

    if !hash_changed {
        return Ok(());
    }

    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;
    let (width, height) = gif_dimensions(&gif_data).unwrap_or((1, 1));
    log::info!("Detected clipboard GIF: {}x{}, {} bytes", width, height, gif_data.len());

    let db = db.lock().unwrap();
    if let Err(e) = db.add_image_item(&gif_data, width, height, "image/gif") {
        log::error!("Failed to save clipboard GIF: {}", e);
    } else {
        log::info!("Saved clipboard GIF: {}x{}", width, height);
    }
    Ok(())
}

enum CapturedClipboard {
    Gif(Vec<u8>),
    ImageFile {
        data: Vec<u8>,
        width: i32,
        height: i32,
        mime: &'static str,
    },
    RgbaImage { bytes: Vec<u8>, width: u32, height: u32 },
    Text(String),
    Files(Vec<std::path::PathBuf>),
    SkipGifFallback,
}

fn save_image_file_if_changed(
    db: &Arc<Mutex<Database>>,
    data: Vec<u8>,
    width: i32,
    height: i32,
    mime: &str,
) -> Result<(), String> {
    if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
        *LAST_CLIPBOARD_HASH.lock().unwrap() = hash_bytes(&data);
        return Ok(());
    }

    let hash = hash_bytes(&data);
    let hash_changed = {
        let h = LAST_CLIPBOARD_HASH.lock().unwrap();
        *h != hash
    };

    if !hash_changed {
        return Ok(());
    }

    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;
    log::info!(
        "Detected clipboard image file: {width}x{height}, {} bytes ({mime})",
        data.len()
    );

    let db = db.lock().unwrap();
    if let Err(e) = db.add_image_item(&data, width, height, mime) {
        log::error!("Failed to save clipboard image file: {e}");
    } else {
        log::info!("Saved clipboard image file: {width}x{height}");
    }
    Ok(())
}

fn save_files_if_changed(
    db: &Arc<Mutex<Database>>,
    paths: Vec<std::path::PathBuf>,
) -> Result<(), String> {
    let paths_str: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let file_path_json = paths_to_storage_json(&paths);

    if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
        *LAST_CLIPBOARD_HASH.lock().unwrap() = hash_file_paths(&paths_str);
        return Ok(());
    }

    let hash = hash_file_paths(&paths_str);
    let hash_changed = {
        let h = LAST_CLIPBOARD_HASH.lock().unwrap();
        *h != hash
    };

    if !hash_changed {
        return Ok(());
    }

    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash.clone();
    let meta = build_file_meta(&paths_str);
    log::info!(
        "Detected clipboard files: {} item(s), {} bytes",
        meta.paths.len(),
        meta.total_size
    );

    let db = db.lock().unwrap();
    if let Ok(false) = db.is_duplicate_file(&file_path_json) {
        if let Err(e) = db.add_file_item(&file_path_json, &meta.display_name, meta.total_size) {
            log::error!("Failed to save clipboard files: {}", e);
        } else {
            log::info!("Saved clipboard files: {}", meta.display_name);
        }
    }
    Ok(())
}

const MAX_IMAGE_DIMENSION: u32 = 4096;
const MAX_IMAGE_PIXELS: u64 = 8_000_000;

fn downscale_rgba_if_needed(bytes: Vec<u8>, width: u32, height: u32) -> Option<(Vec<u8>, u32, u32)> {
    let pixels = width as u64 * height as u64;
    if width <= MAX_IMAGE_DIMENSION && height <= MAX_IMAGE_DIMENSION && pixels <= MAX_IMAGE_PIXELS {
        return Some((bytes, width, height));
    }

    let img = image::RgbaImage::from_raw(width, height, bytes)?;
    let scale = (MAX_IMAGE_DIMENSION as f64 / width.max(height) as f64)
        .min((MAX_IMAGE_PIXELS as f64 / pixels as f64).sqrt());
    let new_w = ((width as f64 * scale).round() as u32).max(1);
    let new_h = ((height as f64 * scale).round() as u32).max(1);
    let resized = image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle);
    log::info!(
        "Downscaled clipboard image from {width}x{height} to {new_w}x{new_h}"
    );
    Some((resized.into_raw(), new_w, new_h))
}


fn capture_clipboard_content() -> Result<CapturedClipboard, String> {
    if SKIP_NEXT_IMAGE.load(Ordering::SeqCst) {
        return Ok(CapturedClipboard::SkipGifFallback);
    }

    #[cfg(target_os = "windows")]
    {
        return match crate::clipboard::win_io::capture_content()? {
            crate::clipboard::win_io::WinCapture::Gif(data) => Ok(CapturedClipboard::Gif(data)),
            crate::clipboard::win_io::WinCapture::ImageFile {
                data,
                width,
                height,
                mime,
            } => Ok(CapturedClipboard::ImageFile {
                data,
                width,
                height,
                mime,
            }),
            crate::clipboard::win_io::WinCapture::Rgba { bytes, width, height } => {
                Ok(CapturedClipboard::RgbaImage { bytes, width, height })
            }
            crate::clipboard::win_io::WinCapture::Text(text) => finalize_text_capture(text),
            crate::clipboard::win_io::WinCapture::Files(paths) => Ok(CapturedClipboard::Files(paths)),
            crate::clipboard::win_io::WinCapture::Empty => Ok(CapturedClipboard::SkipGifFallback),
        };
    }

    #[cfg(not(target_os = "windows"))]
    capture_clipboard_content_non_windows()
}

#[cfg(not(target_os = "windows"))]
fn capture_clipboard_content_non_windows() -> Result<CapturedClipboard, String> {
    let _io_guard = clipboard_io_lock();

    let file_paths = get_clipboard_file_paths();

    if let Some(ref paths) = file_paths {
        if paths.len() == 1 {
            if is_gif_path(&paths[0]) {
                if let Some(gif_data) = read_gif_from_path(&paths[0]) {
                    return Ok(CapturedClipboard::Gif(gif_data));
                }
            } else if is_static_image_path(&paths[0]) {
                if let Some(img) = read_image_file_from_path(&paths[0]) {
                    return Ok(CapturedClipboard::ImageFile {
                        data: img.data,
                        width: img.width,
                        height: img.height,
                        mime: img.mime,
                    });
                }
            }
        }
    }

    if file_paths.is_none() {
        if let Some(gif_data) = try_read_gif_from_hdrop() {
            return Ok(CapturedClipboard::Gif(gif_data));
        }
    }

    if let Some(gif_data) = try_read_gif_from_clipboard_formats() {
        return Ok(CapturedClipboard::Gif(gif_data));
    }

    if clipboard_has_gif_file() {
        let only_gif_files = file_paths.as_ref().is_none_or(|paths| {
            !paths.is_empty() && paths.iter().all(|p| is_gif_path(p))
        });
        if only_gif_files {
            log::warn!("GIF file detected on clipboard but failed to read — skipping static image fallback");
            return Ok(CapturedClipboard::SkipGifFallback);
        }
    }

    if let Some(paths) = file_paths {
        if should_save_as_file_record(&paths) {
            return Ok(CapturedClipboard::Files(paths));
        }
    }

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    match clipboard.get_image() {
        Ok(image_data) => {
            if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
                let hash = hash_bytes(&image_data.bytes);
                *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;
                return Ok(CapturedClipboard::SkipGifFallback);
            }

            let hash = hash_bytes(&image_data.bytes);
            let hash_changed = {
                let h = LAST_CLIPBOARD_HASH.lock().unwrap();
                *h != hash
            };

            if !hash_changed {
                return Ok(CapturedClipboard::SkipGifFallback);
            }

            *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;

            let bytes = image_data.bytes.into_owned();
            let width = image_data.width as u32;
            let height = image_data.height as u32;

            log::info!("Detected clipboard image: {}x{}, {} bytes", width, height, bytes.len());

            return Ok(CapturedClipboard::RgbaImage { bytes, width, height });
        }
        Err(arboard::Error::ContentNotAvailable) => {}
        Err(arboard::Error::ClipboardOccupied) => return Err("ClipboardOccupied".into()),
        Err(_) => {}
    }

    match clipboard.get_text() {
        Ok(text) => finalize_text_capture(text.trim().to_string()),
        Err(arboard::Error::ContentNotAvailable) => Ok(CapturedClipboard::SkipGifFallback),
        Err(arboard::Error::ClipboardOccupied) => Err("ClipboardOccupied".into()),
        Err(e) => Err(e.to_string()),
    }
}

fn finalize_text_capture(normalized: String) -> Result<CapturedClipboard, String> {
    if normalized.is_empty() {
        return Ok(CapturedClipboard::SkipGifFallback);
    }

    let hash = hash_text(&normalized);
    let hash_changed = {
        let h = LAST_CLIPBOARD_HASH.lock().unwrap();
        *h != hash
    };

    if !hash_changed {
        return Ok(CapturedClipboard::SkipGifFallback);
    }

    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;
    Ok(CapturedClipboard::Text(normalized))
}

pub(crate) fn read_and_save_clipboard(db: &Arc<Mutex<Database>>) -> Result<(), String> {
    let captured = capture_clipboard_content()?;

    match captured {
        CapturedClipboard::Gif(gif_data) => save_gif_if_changed(db, gif_data),
        CapturedClipboard::ImageFile {
            data,
            width,
            height,
            mime,
        } => save_image_file_if_changed(db, data, width, height, mime),
        CapturedClipboard::RgbaImage { bytes, width, height } => {
            let Some((bytes, width, height)) = downscale_rgba_if_needed(bytes, width, height) else {
                return Ok(());
            };
            // RGBA 格式，勿再做 BGRA 转换（否则红蓝通道会互换导致偏色）
            if let Some(img) = image::RgbaImage::from_raw(width, height, bytes) {
                let mut png_buf: Vec<u8> = Vec::new();
                if img
                    .write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)
                    .is_ok()
                {
                    let db = db.lock().unwrap();
                    if let Err(e) =
                        db.add_image_item(&png_buf, width as i32, height as i32, "image/png")
                    {
                        log::error!("Failed to save clipboard image: {}", e);
                    } else {
                        log::info!("Saved clipboard image: {}x{}", width, height);
                    }
                }
            }
            Ok(())
        }
        CapturedClipboard::Text(normalized) => {
            let db = db.lock().unwrap();
            if let Ok(false) = db.is_duplicate_text(&normalized) {
                if let Err(e) = db.add_text_item(&normalized) {
                    log::error!("Failed to save clipboard text: {}", e);
                } else {
                    log::info!("Saved clipboard text: {} chars", normalized.len());
                }
            }
            Ok(())
        }
        CapturedClipboard::Files(paths) => save_files_if_changed(db, paths),
        CapturedClipboard::SkipGifFallback => Ok(()),
    }
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}
