use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::clipboard::clipboard_io_lock;
use crate::clipboard::image_io::is_gif_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePathEntry {
    pub path: String,
    pub exists: bool,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClipboardMeta {
    pub paths: Vec<FilePathEntry>,
    pub display_name: String,
    pub total_size: i64,
    pub missing_count: usize,
}

/// 从剪贴板读取文件路径（arboard + HDROP 兜底）
pub fn get_clipboard_file_paths() -> Option<Vec<PathBuf>> {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(files) = clipboard.get().file_list() {
            if !files.is_empty() {
                return Some(files);
            }
        }
    }

    read_paths_from_hdrop()
}

#[cfg(target_os = "windows")]
fn read_paths_from_hdrop() -> Option<Vec<PathBuf>> {
    use clipboard_win::{formats, is_format_avail, raw};

    let _clip = clipboard_win::Clipboard::new_attempts(10).ok()?;
    if !is_format_avail(formats::CF_HDROP) {
        return None;
    }

    let mut paths = Vec::new();
    raw::get_file_list_path(&mut paths).ok()?;
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

#[cfg(not(target_os = "windows"))]
fn read_paths_from_hdrop() -> Option<Vec<PathBuf>> {
    None
}

pub fn paths_to_storage_json(paths: &[PathBuf]) -> String {
    let strings: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    serde_json::to_string(&strings).unwrap_or_else(|_| strings.join("\n"))
}

pub fn parse_stored_paths(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        if let Ok(paths) = serde_json::from_str::<Vec<String>>(trimmed) {
            return paths;
        }
    }

    vec![raw.to_string()]
}

pub fn build_file_meta(paths: &[String]) -> FileClipboardMeta {
    let mut total_size: i64 = 0;
    let mut missing_count = 0;
    let mut entries = Vec::with_capacity(paths.len());

    for path in paths {
        let p = Path::new(path);
        match std::fs::metadata(p) {
            Ok(meta) => {
                let size = meta.len() as i64;
                total_size += size;
                entries.push(FilePathEntry {
                    path: path.clone(),
                    exists: true,
                    size: Some(size),
                });
            }
            Err(_) => {
                missing_count += 1;
                entries.push(FilePathEntry {
                    path: path.clone(),
                    exists: false,
                    size: None,
                });
            }
        }
    }

    let display_name = if paths.len() == 1 {
        Path::new(&paths[0])
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件")
            .to_string()
    } else {
        format!("{} 个文件", paths.len())
    };

    FileClipboardMeta {
        paths: entries,
        display_name,
        total_size,
        missing_count,
    }
}

pub fn existing_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|p| Path::new(p.as_str()).exists())
        .cloned()
        .collect()
}

pub fn hash_file_paths(paths: &[String]) -> String {
    let mut sorted = paths.to_vec();
    sorted.sort();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for path in &sorted {
        hasher.update(path.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// 将文件路径写回系统剪贴板（Windows 原生 CF_HDROP，串行化避免崩溃）
pub fn set_clipboard_files(paths: &[String]) -> Result<(), String> {
    let existing = existing_paths(paths);
    if existing.is_empty() {
        return Err("文件不存在或已被移动，无法复制".into());
    }

    #[cfg(target_os = "windows")]
    {
        use clipboard_win::{raw, Clipboard};

        let _io_guard = clipboard_io_lock();
        let _clip = Clipboard::new_attempts(10)
            .map_err(|e| format!("Failed to open clipboard: {e:?}"))?;

        raw::empty().map_err(|e| format!("Failed to empty clipboard: {e:?}"))?;
        raw::set_file_list(&existing).map_err(|e| format!("Failed to set file list: {e:?}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _io_guard = clipboard_io_lock();
        let path_refs: Vec<&Path> = existing.iter().map(Path::new).collect();
        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        clipboard
            .set()
            .file_list(&path_refs)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 单个 .gif 文件仍走 GIF 动图逻辑；多文件或非 GIF 单文件走 file 记录
pub fn should_save_as_file_record(paths: &[PathBuf]) -> bool {
    if paths.is_empty() {
        return false;
    }
    if paths.len() == 1 && is_gif_path(&paths[0]) {
        return false;
    }
    true
}
