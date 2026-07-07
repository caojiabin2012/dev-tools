use std::path::{Path, PathBuf};

use image::GenericImageView;

pub fn is_gif(data: &[u8]) -> bool {
    data.len() >= 6 && &data[0..3] == b"GIF" && (&data[3..6] == b"87a" || &data[3..6] == b"89a")
}

pub fn gif_dimensions(data: &[u8]) -> Option<(i32, i32)> {
    if !is_gif(data) || data.len() < 10 {
        return None;
    }
    let width = i32::from(u16::from_le_bytes([data[6], data[7]]));
    let height = i32::from(u16::from_le_bytes([data[8], data[9]]));
    if width <= 0 || height <= 0 {
        return None;
    }
    Some((width, height))
}

pub fn image_mime_type(data: &[u8], mime_type: Option<&str>) -> &'static str {
    if is_gif(data) {
        return "image/gif";
    }
    if mime_type == Some("image/gif") {
        return "image/gif";
    }
    if mime_type.is_some_and(|m| m.contains("jpeg") || m.contains("jpg")) {
        return "image/jpeg";
    }
    "image/png"
}

/// 从字节块中提取 GIF（支持嵌入在其他容器格式里的 GIF）
pub fn extract_gif_payload(data: &[u8]) -> Option<Vec<u8>> {
    if is_gif(data) {
        return Some(data.to_vec());
    }
    let scan_limit = data.len().min(8 * 1024 * 1024);
    for i in 0..scan_limit.saturating_sub(6) {
        if is_gif(&data[i..]) {
            return Some(data[i..].to_vec());
        }
    }
    None
}

pub fn read_gif_from_path(path: &Path) -> Option<Vec<u8>> {
    let data = std::fs::read(path).ok()?;
    extract_gif_payload(&data)
}

/// 从剪贴板文件列表读取 GIF（复制 .gif 文件时）
pub fn try_read_gif_from_file_list_paths(paths: &[PathBuf]) -> Option<Vec<u8>> {
    for path in paths {
        if let Some(data) = read_gif_from_path(path) {
            log::info!("Read GIF from copied file: {}", path.display());
            return Some(data);
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
pub fn try_read_gif_from_hdrop() -> Option<Vec<u8>> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn clipboard_has_gif_file() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn try_read_gif_from_clipboard_formats() -> Option<Vec<u8>> {
    None
}

/// 将 PNG/静态图写入剪贴板
pub fn set_clipboard_static_image(data: &[u8]) -> Result<(), String> {
    if is_gif(data) {
        return set_clipboard_gif(data);
    }

    #[cfg(target_os = "windows")]
    {
        return crate::clipboard::clipboard_set_png(data.to_vec());
    }

    #[cfg(not(target_os = "windows"))]
    {
        use crate::clipboard::clipboard_io_lock;

        let _io_guard = clipboard_io_lock();
        let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let bytes = rgba.into_raw();

        let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        let img_data = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: std::borrow::Cow::Owned(bytes),
        };
        clipboard.set_image(img_data).map_err(|e| e.to_string())
    }
}

/// 将 GIF 写入剪贴板（GIF 格式 + PNG 首帧，便于各应用粘贴）
pub fn set_clipboard_gif(data: &[u8]) -> Result<(), String> {
    if !is_gif(data) {
        return Err("Invalid GIF data".into());
    }

    #[cfg(target_os = "windows")]
    {
        return crate::clipboard::clipboard_set_gif(data.to_vec());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("GIF clipboard write is not supported on this platform".into())
    }
}

pub fn is_gif_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gif"))
}

/// 资源管理器复制的单张静态图片（jpg/png 等），应按图片入库而非 file 记录。
pub fn is_static_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "bmp"
            )
        })
}

pub fn mime_from_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "webp" => "image/webp",
        Some(ext) if ext == "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

const MAX_IMAGE_FILE_BYTES: usize = 64 * 1024 * 1024;

pub struct ImageFileData {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub mime: &'static str,
}

/// 从复制的单个图片文件路径读取原图（保留 jpeg/png 等原始字节）。
pub fn read_image_file_from_path(path: &Path) -> Option<ImageFileData> {
    let data = std::fs::read(path).ok()?;
    if data.len() > MAX_IMAGE_FILE_BYTES {
        log::warn!(
            "Image file too large ({} bytes): {}",
            data.len(),
            path.display()
        );
        return None;
    }
    let img = image::load_from_memory(&data).ok()?;
    let (w, h) = img.dimensions();
    log::info!(
        "Read image from copied file: {} ({}x{}, {} bytes)",
        path.display(),
        w,
        h,
        data.len()
    );
    Some(ImageFileData {
        data,
        width: w as i32,
        height: h as i32,
        mime: mime_from_path(path),
    })
}
