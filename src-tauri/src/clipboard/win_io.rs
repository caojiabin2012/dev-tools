//! Windows 剪贴板统一入口（纯 clipboard_win / Win32）。
//! **仅** clipboard worker 线程调用，禁止在其他线程直接 OpenClipboard。

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use clipboard_win::{formats, is_format_avail, raw, register_format, Clipboard, EnumFormats};

use crate::clipboard::file_io::should_save_as_file_record;
use crate::clipboard::image_io::{
    extract_gif_payload, is_gif_path, is_static_image_path, read_gif_from_path,
    read_image_file_from_path,
};

pub const MAX_RGBA_BYTES: usize = 64 * 1024 * 1024;

const MAX_OPEN_ATTEMPTS: i32 = 5;
const OPEN_RETRY_SLEEP_MS: u64 = 5;

pub enum WinCapture {
    Gif(Vec<u8>),
    ImageFile {
        data: Vec<u8>,
        width: i32,
        height: i32,
        mime: &'static str,
    },
    Rgba {
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    },
    Text(String),
    Files(Vec<PathBuf>),
    Empty,
}

pub fn open_clipboard() -> Result<Clipboard, String> {
    for attempt in 0..MAX_OPEN_ATTEMPTS {
        match Clipboard::new_attempts(1) {
            Ok(clip) => return Ok(clip),
            Err(e) => {
                if attempt + 1 >= MAX_OPEN_ATTEMPTS {
                    return Err(format!("Failed to open clipboard: {e:?}"));
                }
                thread::sleep(Duration::from_millis(OPEN_RETRY_SLEEP_MS));
            }
        }
    }
    Err("Failed to open clipboard".into())
}

pub fn sequence_number() -> u32 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() }
}

/// 单次 OpenClipboard 会话内读取并分类（GIF 文件 → GIF 格式 → 多文件 → 图片 → 文本）。
pub fn capture_content() -> Result<WinCapture, String> {
    let _clip = open_clipboard()?;

    let file_paths = read_file_paths();

    if let Some(ref paths) = file_paths {
        if paths.len() == 1 {
            if is_gif_path(&paths[0]) {
                if let Some(gif_data) = read_gif_from_path(&paths[0]) {
                    return Ok(WinCapture::Gif(gif_data));
                }
            } else if is_static_image_path(&paths[0]) {
                if let Some(img) = read_image_file_from_path(&paths[0]) {
                    return Ok(WinCapture::ImageFile {
                        data: img.data,
                        width: img.width,
                        height: img.height,
                        mime: img.mime,
                    });
                }
            }
        }
    }

    if let Some(gif_data) = read_gif_from_formats() {
        return Ok(WinCapture::Gif(gif_data));
    }

    if has_gif_file_on_clipboard() {
        let only_gif = file_paths.as_ref().is_none_or(|paths| {
            !paths.is_empty() && paths.iter().all(|p| is_gif_path(p))
        });
        if only_gif {
            log::warn!("GIF on clipboard but read failed — skip static image fallback");
            return Ok(WinCapture::Empty);
        }
    }

    if let Some(paths) = file_paths {
        if should_save_as_file_record(&paths) {
            return Ok(WinCapture::Files(paths));
        }
    }

    if let Some((bytes, width, height)) = read_image_rgba() {
        let byte_len = bytes.len();
        if byte_len > MAX_RGBA_BYTES {
            log::warn!("Clipboard image too large ({byte_len} bytes RGBA), skipping");
        } else {
            log::info!("Detected clipboard image: {width}x{height}, {byte_len} bytes RGBA");
            return Ok(WinCapture::Rgba {
                bytes,
                width,
                height,
            });
        }
    }

    if let Some(text) = read_unicode_text() {
        return Ok(WinCapture::Text(text));
    }

    Ok(WinCapture::Empty)
}

pub fn read_unicode_text() -> Option<String> {
    if !is_format_avail(formats::CF_UNICODETEXT) {
        return None;
    }
    let mut buf = Vec::new();
    raw::get_string(&mut buf).ok()?;
    String::from_utf8(buf)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn read_file_paths() -> Option<Vec<PathBuf>> {
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

fn read_gif_from_formats() -> Option<Vec<u8>> {
    for name in ["GIF", "gif", "GIF Format"] {
        if let Some(format) = register_format(name) {
            if !is_format_avail(format.get()) {
                continue;
            }
            let mut data = Vec::new();
            if raw::get_vec(format.get(), &mut data).is_ok() {
                if let Some(gif) = extract_gif_payload(&data) {
                    log::info!("Read GIF from clipboard format '{name}'");
                    return Some(gif);
                }
            }
        }
    }

    for format_id in EnumFormats::new() {
        if !is_format_avail(format_id) {
            continue;
        }
        let mut data = Vec::new();
        if raw::get_vec(format_id, &mut data).is_err() {
            continue;
        }
        if let Some(gif) = extract_gif_payload(&data) {
            log::info!("Read embedded GIF from clipboard format id {format_id}");
            return Some(gif);
        }
    }
    None
}

fn has_gif_file_on_clipboard() -> bool {
    read_file_paths()
        .is_some_and(|paths| paths.iter().any(|p| is_gif_path(p)))
}

pub fn read_image_rgba() -> Option<(Vec<u8>, u32, u32)> {
    if let Some(png_format) = register_format("PNG") {
        if is_format_avail(png_format.get()) {
            let mut data = Vec::new();
            if raw::get_vec(png_format.get(), &mut data).is_ok() && !data.is_empty() {
                if let Ok(img) = image::load_from_memory(&data) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    return Some((rgba.into_raw(), w, h));
                }
            }
        }
    }

    if is_format_avail(formats::CF_DIB) {
        let mut dib = Vec::new();
        if raw::get_vec(formats::CF_DIB, &mut dib).is_ok() {
            if let Some(result) = dib32_to_rgba(&dib) {
                return Some(result);
            }
        }
    }

    if is_format_avail(formats::CF_BITMAP) {
        let mut bmp = Vec::new();
        if raw::get_bitmap(&mut bmp).is_ok() && !bmp.is_empty() {
            if let Ok(img) = image::load_from_memory(&bmp) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                return Some((rgba.into_raw(), w, h));
            }
        }
    }

    None
}

fn dib32_to_rgba(dib: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    if dib.len() < 40 {
        return None;
    }
    let header_size = u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]) as usize;
    let width = i32::from_le_bytes([dib[4], dib[5], dib[6], dib[7]]);
    let height = i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]);
    let bpp = u16::from_le_bytes([dib[14], dib[15]]);

    if width <= 0 || bpp != 32 || dib.len() <= header_size {
        return None;
    }

    let w = width as u32;
    let abs_h = height.unsigned_abs();
    let row_bytes = ((width * 4 + 3) / 4) * 4;
    let bits = &dib[header_size..];
    let needed = row_bytes as usize * abs_h as usize;
    if bits.len() < needed {
        return None;
    }

    let mut rgba = Vec::with_capacity((w * abs_h * 4) as usize);
    for row in 0..abs_h {
        let src_row = if height < 0 { row } else { abs_h - 1 - row };
        let start = src_row as usize * row_bytes as usize;
        for col in 0..w {
            let i = start + col as usize * 4;
            rgba.push(bits[i + 2]);
            rgba.push(bits[i + 1]);
            rgba.push(bits[i]);
            rgba.push(bits[i + 3]);
        }
    }
    Some((rgba, w, abs_h))
}

pub fn write_text(text: &str) -> Result<(), String> {
    let _clip = open_clipboard()?;
    raw::empty().map_err(|e| format!("Failed to empty clipboard: {e:?}"))?;
    raw::set_string(text).map_err(|e| format!("Failed to set clipboard text: {e:?}"))
}

pub fn write_png(data: &[u8]) -> Result<(), String> {
    let _clip = open_clipboard()?;
    raw::empty().map_err(|e| format!("Failed to empty clipboard: {e:?}"))?;
    let png_format = register_format("PNG").ok_or("Failed to register PNG format")?;
    raw::set_without_clear(png_format.get(), data)
        .map_err(|e| format!("Failed to set PNG on clipboard: {e:?}"))
}

pub fn write_gif(data: &[u8]) -> Result<(), String> {
    use std::io::Cursor;

    if !crate::clipboard::image_io::is_gif(data) {
        return Err("Invalid GIF data".into());
    }
    let _clip = open_clipboard()?;
    raw::empty().map_err(|e| format!("Failed to empty clipboard: {e:?}"))?;
    let gif_format = register_format("GIF").ok_or("Failed to register GIF format")?;
    raw::set_without_clear(gif_format.get(), data)
        .map_err(|e| format!("Failed to set GIF on clipboard: {e:?}"))?;

    if let Ok(img) = image::load_from_memory(data) {
        let mut png_buf = Vec::new();
        if img
            .write_to(&mut Cursor::new(&mut png_buf), image::ImageFormat::Png)
            .is_ok()
        {
            if let Some(png_format) = register_format("PNG") {
                let _ = raw::set_without_clear(png_format.get(), &png_buf);
            }
        }
    }
    Ok(())
}

pub fn write_files(paths: &[String]) -> Result<(), String> {
    use crate::clipboard::file_io::existing_paths;

    let existing = existing_paths(paths);
    if existing.is_empty() {
        return Err("文件不存在或已被移动，无法复制".into());
    }
    let _clip = open_clipboard()?;
    raw::empty().map_err(|e| format!("Failed to empty clipboard: {e:?}"))?;
    raw::set_file_list(&existing).map_err(|e| format!("Failed to set file list: {e:?}"))
}

/// 读取 PNG 字节并计算 RGBA hash（与监听器读回 PNG 解码后一致）。
pub fn hash_png_as_rgba(png_data: &[u8]) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let img = image::load_from_memory(png_data).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let mut hasher = Sha256::new();
    hasher.update(rgba.as_raw());
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}
