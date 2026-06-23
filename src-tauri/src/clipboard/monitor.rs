use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::clipboard::database::Database;

pub struct ClipboardMonitor {
    db: Arc<Mutex<Database>>,
    last_text: Arc<Mutex<String>>,
    running: Arc<Mutex<bool>>,
}

impl ClipboardMonitor {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            last_text: Arc::new(Mutex::new(String::new())),
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
        let last_text = self.last_text.clone();
        let running = self.running.clone();

        std::thread::spawn(move || {
            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    return;
                }
            };

            while *running.lock().unwrap() {
                if let Ok(text) = clipboard.get_text() {
                    let mut last = last_text.lock().unwrap();
                    if text != *last && !text.is_empty() {
                        *last = text.clone();
                        drop(last);

                        let db = db.lock().unwrap();
                        if let Ok(false) = db.is_duplicate_text(&text) {
                            if let Err(e) = db.add_text_item(&text) {
                                log::error!("Failed to save clipboard text: {}", e);
                            } else {
                                log::info!("Saved clipboard text: {} chars", text.len());
                            }
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}
