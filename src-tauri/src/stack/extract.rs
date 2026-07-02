use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

pub fn extract_zip(zip_path: &Path, target_base: &Path) -> Result<(), String> {
    fs::create_dir_all(target_base).map_err(|e| e.to_string())?;
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("无法打开 zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let out_path = target_base.join(name);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 在 base 下递归查找包含指定可执行文件的目录
pub fn find_home_with_binary(base: &Path, relative_bin: &[&str]) -> Result<PathBuf, String> {
    search_binary_home(base, relative_bin, 5).ok_or_else(|| {
        format!(
            "未找到 {} 于 {}（请确认 zip 包完整且为 Windows x64 NTS 版本）",
            relative_bin.join("/"),
            base.display()
        )
    })
}

fn search_binary_home(base: &Path, relative_bin: &[&str], depth: u32) -> Option<PathBuf> {
    if binary_exists(base, relative_bin) {
        return Some(base.to_path_buf());
    }
    if depth == 0 {
        return None;
    }
    let entries = fs::read_dir(base).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(found) = search_binary_home(&path, relative_bin, depth.saturating_sub(1)) {
            return Some(found);
        }
    }
    None
}

pub fn find_php_home(base: &Path) -> Result<PathBuf, String> {
    find_home_with_binary(base, &["php-cgi.exe"])
}

/// 查找 MySQL / MariaDB 安装目录（兼容 mysqld.exe 与 mariadbd.exe）
pub fn find_mysql_home(base: &Path) -> Result<PathBuf, String> {
    for rel in [["bin", "mariadbd.exe"], ["bin", "mysqld.exe"]] {
        if let Ok(home) = find_home_with_binary(base, &rel) {
            return Ok(home);
        }
    }
    Err(format!(
        "未找到 bin/mysqld.exe 或 bin/mariadbd.exe 于 {}",
        base.display()
    ))
}

pub fn binary_exists(base: &Path, parts: &[&str]) -> bool {
    let mut path = base.to_path_buf();
    for part in parts {
        path = path.join(part);
    }
    path.exists()
}

pub fn binary_path(base: &Path, parts: &[&str]) -> PathBuf {
    let mut path = base.to_path_buf();
    for part in parts {
        path = path.join(part);
    }
    path
}
