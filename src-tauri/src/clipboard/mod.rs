pub mod database;
pub mod commands;
pub mod monitor;
pub mod image_io;

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use once_cell::sync::Lazy;

pub static LAST_CLIPBOARD_HASH: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));
pub static SKIP_NEXT_IMAGE: AtomicBool = AtomicBool::new(false);

pub use commands::*;
pub use database::Database;
pub use monitor::ClipboardMonitor;
