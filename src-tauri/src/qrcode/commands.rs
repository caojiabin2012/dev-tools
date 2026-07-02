use std::io::Cursor;
use std::sync::{Arc, Mutex};

use image::{ImageFormat, Luma};
use qrcode::QrCode;
use rqrr::PreparedImage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::clipboard::image_io::set_clipboard_static_image;
use crate::clipboard::{OwnClipboardWriteGuard, LAST_CLIPBOARD_HASH};
use crate::qrcode::database::{QrDecodeItem, QrGenerateDetail, QrGeneratePreview, QrcodeDatabase};

pub struct QrcodeDbState {
    pub db: Arc<Mutex<QrcodeDatabase>>,
}

const MAX_QR_TEXT_LEN: usize = 2048;
const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;

fn hash_rgba_png(png_data: &[u8]) -> Result<String, String> {
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

fn decode_qr_bytes(image_data: &[u8]) -> Result<String, String> {
    if image_data.is_empty() {
        return Err("图片为空".into());
    }
    if image_data.len() > MAX_IMAGE_BYTES {
        return Err("图片过大，请选择 12MB 以内的文件".into());
    }

    let dynamic = image::load_from_memory(image_data).map_err(|e| format!("无法读取图片: {e}"))?;
    let luma = dynamic.to_luma8();
    let mut prepared = PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();

    for grid in grids {
        if let Ok((_, content)) = grid.decode() {
            if !content.is_empty() {
                return Ok(content);
            }
        }
    }

    Err("未识别到二维码，请换一张更清晰的图片".into())
}

fn generate_qr_png_bytes(text: &str) -> Result<Vec<u8>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("请输入要编码的内容".into());
    }
    if trimmed.len() > MAX_QR_TEXT_LEN {
        return Err(format!("内容过长，请控制在 {MAX_QR_TEXT_LEN} 字符以内"));
    }

    let code = QrCode::new(trimmed.as_bytes()).map_err(|e| format!("生成失败: {e}"))?;
    let img = code
        .render::<Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(280, 280)
        .max_dimensions(512, 512)
        .build();

    let mut png = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|e| format!("导出 PNG 失败: {e}"))?;
    Ok(png)
}

#[derive(Debug, Deserialize)]
pub struct DecodeQrParams {
    pub image_data: Vec<u8>,
    pub file_name: String,
}

#[tauri::command]
pub fn decode_qr_and_save(
    state: State<'_, QrcodeDbState>,
    params: DecodeQrParams,
) -> Result<QrDecodeItem, String> {
    let text = decode_qr_bytes(&params.image_data)?;
    let file_name = if params.file_name.trim().is_empty() {
        "unknown".to_string()
    } else {
        params.file_name
    };

    let id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.add_decode(&text, &file_name)
            .map_err(|e| e.to_string())?
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_decode(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "保存解析记录失败".to_string())
}

#[derive(Debug, Deserialize)]
pub struct ListQrItemsParams {
    pub time_filter: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListQrDecodeResult {
    pub items: Vec<QrDecodeItem>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ListQrGenerateResult {
    pub items: Vec<QrGeneratePreview>,
    pub total: i64,
}

#[tauri::command]
pub fn list_qr_decode_items(
    state: State<'_, QrcodeDbState>,
    params: ListQrItemsParams,
) -> Result<ListQrDecodeResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let time_filter = params.time_filter.as_deref();
    let search_query = params.search_query.as_deref();
    let items = db.list_decode(time_filter, search_query).map_err(|e| e.to_string())?;
    let total = db.count_decode(time_filter, search_query).map_err(|e| e.to_string())?;
    Ok(ListQrDecodeResult { items, total })
}

#[tauri::command]
pub fn get_qr_decode_item(
    state: State<'_, QrcodeDbState>,
    id: i64,
) -> Result<Option<QrDecodeItem>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_decode(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_qr_decode_item(state: State<'_, QrcodeDbState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_decode(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_qr_decode_items(state: State<'_, QrcodeDbState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_decode().map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct GenerateQrParams {
    pub text: String,
}

#[tauri::command]
pub fn generate_qr_and_save(
    state: State<'_, QrcodeDbState>,
    params: GenerateQrParams,
) -> Result<QrGenerateDetail, String> {
    let trimmed = params.text.trim().to_string();
    let png = generate_qr_png_bytes(&trimmed)?;

    let id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.add_generate(&trimmed, &png)
            .map_err(|e| e.to_string())?
    };

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let detail = db
        .get_generate(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "保存生成记录失败".to_string())?;
    Ok(detail)
}

#[tauri::command]
pub fn list_qr_generate_items(
    state: State<'_, QrcodeDbState>,
    params: ListQrItemsParams,
) -> Result<ListQrGenerateResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let time_filter = params.time_filter.as_deref();
    let search_query = params.search_query.as_deref();
    let items = db
        .list_generate(time_filter, search_query)
        .map_err(|e| e.to_string())?;
    let total = db
        .count_generate(time_filter, search_query)
        .map_err(|e| e.to_string())?;
    Ok(ListQrGenerateResult { items, total })
}

#[tauri::command]
pub fn get_qr_generate_item(
    state: State<'_, QrcodeDbState>,
    id: i64,
) -> Result<Option<QrGenerateDetail>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_generate(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_qr_generate_item(state: State<'_, QrcodeDbState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_generate(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_qr_generate_items(state: State<'_, QrcodeDbState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_generate().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_qr_generate_to_clipboard(
    state: State<'_, QrcodeDbState>,
    id: i64,
) -> Result<(), String> {
    let png = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let detail = db
            .get_generate(id)
            .map_err(|e| e.to_string())?
            .ok_or("记录不存在")?;
        detail.png_data
    };

    let _write_guard = OwnClipboardWriteGuard::begin();
    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash_rgba_png(&png)?;
    set_clipboard_static_image(&png)
}

#[tauri::command]
pub fn save_qr_generate_file(
    state: State<'_, QrcodeDbState>,
    id: i64,
    default_name: Option<String>,
) -> Result<Option<String>, String> {
    let png = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let detail = db
            .get_generate(id)
            .map_err(|e| e.to_string())?
            .ok_or("记录不存在")?;
        detail.png_data
    };

    if png.is_empty() {
        return Err("没有可保存的图片".into());
    }

    let name = default_name
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "qrcode.png".to_string());
    let path = rfd::FileDialog::new()
        .set_file_name(&name)
        .add_filter("PNG 图片", &["png"])
        .save_file();

    match path {
        Some(path) => {
            std::fs::write(&path, png).map_err(|e| format!("保存失败: {e}"))?;
            Ok(Some(path.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn decode_qr_from_image(image_data: Vec<u8>) -> Result<String, String> {
    decode_qr_bytes(&image_data)
}

#[tauri::command]
pub fn generate_qr_code_png(text: String) -> Result<Vec<u8>, String> {
    generate_qr_png_bytes(&text)
}

#[tauri::command]
pub fn copy_png_to_clipboard(png_data: Vec<u8>) -> Result<(), String> {
    if png_data.is_empty() {
        return Err("没有可复制的图片".into());
    }
    let _write_guard = OwnClipboardWriteGuard::begin();
    *LAST_CLIPBOARD_HASH.lock().unwrap() = hash_rgba_png(&png_data)?;
    set_clipboard_static_image(&png_data)
}

#[tauri::command]
pub fn save_png_file(png_data: Vec<u8>, default_name: Option<String>) -> Result<Option<String>, String> {
    if png_data.is_empty() {
        return Err("没有可保存的图片".into());
    }
    let name = default_name
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "qrcode.png".to_string());
    let path = rfd::FileDialog::new()
        .set_file_name(&name)
        .add_filter("PNG 图片", &["png"])
        .save_file();
    match path {
        Some(path) => {
            std::fs::write(&path, png_data).map_err(|e| format!("保存失败: {e}"))?;
            Ok(Some(path.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}
