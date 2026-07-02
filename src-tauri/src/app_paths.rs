use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub const APP_NAME: &str = "DevTools";
pub const APP_DATA_DIR: &str = "dev-tools";
const LEGACY_APP_DATA_DIR: &str = "tool-kit";

/// 应用数据目录（`%LOCALAPPDATA%/dev-tools`），首次启动时从旧目录 `tool-kit` 迁移。
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_dir = base.join(APP_DATA_DIR);
    if !new_dir.exists() {
        let legacy = base.join(LEGACY_APP_DATA_DIR);
        if legacy.exists() {
            let _ = std::fs::rename(&legacy, &new_dir);
        }
    }
    std::fs::create_dir_all(&new_dir).ok();
    new_dir
}

pub fn logs_dir() -> PathBuf {
    let dir = app_data_dir().join("logs");
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// 追加写入诊断日志（`crash.log` 等），便于崩溃后排查。
pub fn append_diagnostic(relative_path: &str, section: &str, message: &str) {
    let path = app_data_dir().join(relative_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("\n=== {timestamp} [{section}] ===\n{message}\n");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(entry.as_bytes());
    }
}
