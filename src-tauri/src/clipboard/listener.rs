//! macOS / Linux 剪贴板变更监听（非 Windows）。
//!
//! - Linux：X11 XFixes 选择事件，变更时才回调（非定时读剪贴板）。
//! - macOS：系统无公开变更通知，使用 NSPasteboard `changeCount` 检测；
//!   仅在计数变化时读取内容，不做全量轮询。

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clipboard_master::{CallbackResult, ClipboardHandler, Master};

use crate::clipboard::{schedule_clipboard_read, SKIP_NEXT_IMAGE};

struct ClipboardChangeHandler {
    running: Arc<Mutex<bool>>,
}

impl ClipboardHandler for ClipboardChangeHandler {
    /// Linux 由 XFixes 事件驱动；此间隔仅用于 shutdown 通道与事件循环超时。
    /// macOS 仅比较 changeCount（整数），间隔越短响应越快，不会每次读剪贴板内容。
    fn sleep_interval(&self) -> Duration {
        #[cfg(target_os = "macos")]
        {
            Duration::from_millis(80)
        }
        #[cfg(target_os = "linux")]
        {
            Duration::from_millis(250)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Duration::from_millis(500)
        }
    }

    fn on_clipboard_change(&mut self) -> CallbackResult {
        if !*self.running.lock().unwrap() {
            return CallbackResult::Stop;
        }

        if SKIP_NEXT_IMAGE.load(std::sync::atomic::Ordering::SeqCst) {
            return CallbackResult::Next;
        }

        schedule_clipboard_read();
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        log::warn!("Clipboard listener error: {}", error);
        CallbackResult::Next
    }
}

pub fn start(running: Arc<Mutex<bool>>) {
    std::thread::spawn(move || {
        let handler = ClipboardChangeHandler { running: running.clone() };

        let mut master = match Master::new(handler) {
            Ok(master) => master,
            Err(e) => {
                log::error!("Failed to create clipboard listener: {}", e);
                return;
            }
        };

        log::info!("Clipboard native listener started.");

        loop {
            if !*running.lock().unwrap() {
                break;
            }

            match master.run() {
                Ok(()) => {
                    log::debug!("Clipboard listener exited normally, restarting...");
                }
                Err(e) => {
                    log::error!("Clipboard listener error: {e}");
                    #[cfg(target_os = "linux")]
                    log::warn!(
                        "Linux 剪贴板监听需要 X11 或 XWayland；纯 Wayland 会话可能无法使用 XFixes 监听"
                    );
                }
            }

            if !*running.lock().unwrap() {
                break;
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        log::info!("Clipboard native listener stopped.");
    });
}
