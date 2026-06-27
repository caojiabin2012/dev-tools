pub mod database;
pub mod commands;
pub mod monitor;
pub mod image_io;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use once_cell::sync::Lazy;

pub static LAST_CLIPBOARD_HASH: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));
pub static SKIP_NEXT_IMAGE: AtomicBool = AtomicBool::new(false);

/// 标记应用自身正在写剪贴板，避免监听器与写操作并发访问导致 Windows 崩溃。
pub struct OwnClipboardWriteGuard;

impl OwnClipboardWriteGuard {
    pub fn begin() -> Self {
        SKIP_NEXT_IMAGE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for OwnClipboardWriteGuard {
    fn drop(&mut self) {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(300));
            SKIP_NEXT_IMAGE.store(false, Ordering::SeqCst);
        });
    }
}

pub use commands::*;
pub use database::Database;
pub use monitor::ClipboardMonitor;
