use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use sha2::{Sha256, Digest};
use crate::clipboard::database::Database;
use crate::clipboard::{LAST_CLIPBOARD_HASH, SKIP_NEXT_IMAGE};

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

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
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

fn read_and_save_clipboard(db: &Arc<Mutex<Database>>) -> Result<(), String> {
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

                let mut rgba_bytes = Vec::with_capacity(bytes.len());
                for chunk in bytes.chunks(4) {
                    if chunk.len() == 4 {
                        rgba_bytes.push(chunk[2]);
                        rgba_bytes.push(chunk[1]);
                        rgba_bytes.push(chunk[0]);
                        rgba_bytes.push(chunk[3]);
                    }
                }

                if let Some(img) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, rgba_bytes) {
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
