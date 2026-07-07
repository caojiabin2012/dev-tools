pub mod database;
pub mod commands;
pub mod monitor;
pub mod image_io;
pub mod file_io;
#[cfg(target_os = "windows")]
pub(crate) mod win_io;
mod worker;

#[cfg(not(target_os = "windows"))]
mod listener;

use std::sync::{Arc, Mutex};
#[cfg(not(target_os = "windows"))]
use std::sync::MutexGuard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use once_cell::sync::Lazy;

pub static LAST_CLIPBOARD_HASH: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));
pub static SKIP_NEXT_IMAGE: AtomicBool = AtomicBool::new(false);
/// 应用正在写剪贴板；worker 在此期间跳过读取。
pub static CLIPBOARD_WRITING: AtomicBool = AtomicBool::new(false);

/// 非 Windows：arboard 与剪贴板 API 混用时串行化。Windows 由 clipboard worker 单线程负责。
#[cfg(not(target_os = "windows"))]
pub static CLIPBOARD_IO_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(not(target_os = "windows"))]
pub fn clipboard_io_lock() -> MutexGuard<'static, ()> {
    CLIPBOARD_IO_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub use worker::{schedule_clipboard_read, start_clipboard_worker};

#[cfg(target_os = "windows")]
pub use worker::{
    clipboard_get_text, clipboard_set_files, clipboard_set_gif, clipboard_set_png,
    clipboard_set_text,
};

/// 标记应用自身正在写剪贴板，避免监听器与写操作并发访问导致 Windows 崩溃。
pub struct OwnClipboardWriteGuard;

impl OwnClipboardWriteGuard {
    pub fn begin() -> Self {
        SKIP_NEXT_IMAGE.store(true, Ordering::SeqCst);
        CLIPBOARD_WRITING.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for OwnClipboardWriteGuard {
    fn drop(&mut self) {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(500));
            SKIP_NEXT_IMAGE.store(false, Ordering::SeqCst);
            CLIPBOARD_WRITING.store(false, Ordering::SeqCst);
        });
    }
}

pub use commands::*;
pub use database::Database;
pub use monitor::ClipboardMonitor;
