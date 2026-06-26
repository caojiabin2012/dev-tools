use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use sha2::{Sha256, Digest};
use crate::clipboard::database::Database;
use crate::clipboard::{LAST_CLIPBOARD_HASH, SKIP_NEXT_IMAGE};
use crate::clipboard::image_io::{
    gif_dimensions, try_read_gif_from_file_list_paths, try_read_gif_from_clipboard_formats,
    try_read_gif_from_hdrop, clipboard_has_gif_file,
};

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
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    WM_CLIPBOARDUPDATE, WNDCLASSW, MSG, GWLP_USERDATA, SetWindowLongPtrW, GetWindowLongPtrW,
    HWND_MESSAGE,
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
            self.start_polling(db, running);
        }
    }

    #[cfg(target_os = "windows")]
    fn start_windows_listener(&self, db: Arc<Mutex<Database>>, running: Arc<Mutex<bool>>) {
        std::thread::spawn(move || {
            unsafe {
                let instance = GetModuleHandleW(None).unwrap();
                let window_class = "ToolKitClipboardListener";
                let window_class_w: Vec<u16> = window_class.encode_utf16().chain(std::iter::once(0)).collect();

                let wnd_class = WNDCLASSW {
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: instance.into(),
                    lpszClassName: PCWSTR(window_class_w.as_ptr()),
                    ..Default::default()
                };

                if RegisterClassW(&wnd_class) == 0 {
                    log::error!("Failed to register clipboard listener window class");
                    return;
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
                        return;
                    }
                };

                let shared = Box::new(ClipboardShared { db, running });
                let ptr = Box::into_raw(shared);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);

                if let Err(e) = AddClipboardFormatListener(hwnd) {
                    log::error!("Failed to add clipboard listener: {:?}", e);
                    let _ = Box::from_raw(ptr);
                    return;
                }

                log::info!("Clipboard event-driven listener started.");

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    DispatchMessageW(&msg);
                }

                let _ = RemoveClipboardFormatListener(hwnd);
                let _ = Box::from_raw(ptr);
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    fn start_polling(&self, db: Arc<Mutex<Database>>, running: Arc<Mutex<bool>>) {
        std::thread::spawn(move || {
            while *running.lock().unwrap() {
                if let Err(e) = read_and_save_clipboard(&db) {
                    log::trace!("Clipboard read error: {}", e);
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}

struct ClipboardShared {
    db: Arc<Mutex<Database>>,
    running: Arc<Mutex<bool>>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ptr != 0 {
                let shared = &*(ptr as *const ClipboardShared);
                if *shared.running.lock().unwrap() {
                    if let Err(e) = read_and_save_clipboard(&shared.db) {
                        log::trace!("Clipboard read error: {}", e);
                    }
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

fn read_and_save_clipboard(db: &Arc<Mutex<Database>>) -> Result<(), String> {
    // 1. 复制 .gif 文件：优先从文件列表读取原文件（保留动画）
    {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        if let Ok(files) = clipboard.get().file_list() {
            if let Some(gif_data) = try_read_gif_from_file_list_paths(&files) {
                return save_gif_if_changed(db, gif_data);
            }
        }
    }

    // 1b. HDROP 兜底（arboard 有时读不到文件列表）
    if let Some(gif_data) = try_read_gif_from_hdrop() {
        return save_gif_if_changed(db, gif_data);
    }

    // 2. 剪贴板上的 GIF 格式 / 嵌入 GIF 数据
    if let Some(gif_data) = try_read_gif_from_clipboard_formats() {
        return save_gif_if_changed(db, gif_data);
    }

    // 剪贴板上是 .gif 文件但读取失败时，不要用静态缩略图覆盖
    if clipboard_has_gif_file() {
        log::warn!("GIF file detected on clipboard but failed to read — skipping static image fallback");
        return Ok(());
    }

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    match clipboard.get_image() {
        Ok(image_data) => {
            if SKIP_NEXT_IMAGE.swap(false, Ordering::SeqCst) {
                let hash = hash_bytes(&image_data.bytes);
                *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;
                return Ok(());
            }

            let hash = hash_bytes(&image_data.bytes);
            let hash_changed = {
                let h = LAST_CLIPBOARD_HASH.lock().unwrap();
                *h != hash
            };

            if hash_changed {
                *LAST_CLIPBOARD_HASH.lock().unwrap() = hash;

                let bytes = image_data.bytes.into_owned();
                let width = image_data.width as u32;
                let height = image_data.height as u32;

                log::info!("Detected clipboard image: {}x{}, {} bytes", width, height, bytes.len());

                // arboard 已返回 RGBA 格式，勿再做 BGRA 转换（否则红蓝通道会互换导致偏色）
                if let Some(img) = image::RgbaImage::from_raw(width, height, bytes) {
                    let mut png_buf: Vec<u8> = Vec::new();
                    if img.write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png).is_ok() {
                        let db = db.lock().unwrap();
                        if let Err(e) = db.add_image_item(&png_buf, width as i32, height as i32, "image/png") {
                            log::error!("Failed to save clipboard image: {}", e);
                        } else {
                            log::info!("Saved clipboard image: {}x{}", width, height);
                        }
                    }
                }
            }
            return Ok(());
        }
        Err(arboard::Error::ContentNotAvailable) => {}
        Err(arboard::Error::ClipboardOccupied) => return Ok(()),
        Err(_) => {}
    }

    match clipboard.get_text() {
        Ok(text) => {
            let normalized = text.trim().to_string();
            if normalized.is_empty() {
                return Ok(());
            }

            let hash = hash_text(&normalized);
            let hash_changed = {
                let h = LAST_CLIPBOARD_HASH.lock().unwrap();
                *h != hash
            };

            if hash_changed {
                *LAST_CLIPBOARD_HASH.lock().unwrap() = hash.clone();

                let db = db.lock().unwrap();
                if let Ok(false) = db.is_duplicate_text(&normalized) {
                    if let Err(e) = db.add_text_item(&normalized) {
                        log::error!("Failed to save clipboard text: {}", e);
                    } else {
                        log::info!("Saved clipboard text: {} chars", normalized.len());
                    }
                }
            }
        }
        Err(arboard::Error::ContentNotAvailable) => {}
        Err(arboard::Error::ClipboardOccupied) => {}
        Err(_) => {}
    }

    Ok(())
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
