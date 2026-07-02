use crate::stack::types::DbEngine;

#[derive(Debug, Clone, Copy)]
pub struct VersionEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    pub mirror_urls: &'static [&'static str],
    pub engine: Option<DbEngine>,
}

macro_rules! php_entry {
    ($id:expr, $label:expr, $ver:expr, $toolchain:expr) => {
        VersionEntry {
            id: $id,
            label: $label,
            filename: concat!("php-", $ver, "-nts-Win32-", $toolchain, "-x64.zip"),
            url: concat!(
                "https://windows.php.net/downloads/releases/php-",
                $ver,
                "-nts-Win32-",
                $toolchain,
                "-x64.zip"
            ),
            mirror_urls: &[
                concat!(
                    "https://downloads.php.net/~windows/releases/php-",
                    $ver,
                    "-nts-Win32-",
                    $toolchain,
                    "-x64.zip"
                ),
            ],
            engine: None,
        }
    };
}

macro_rules! redis_entry {
    ($id:expr, $label:expr, $tag:expr, $file:literal) => {
        VersionEntry {
            id: $id,
            label: $label,
            filename: $file,
            url: concat!(
                "https://ghfast.top/https://github.com/redis-windows/redis-windows/releases/download/",
                $tag,
                "/",
                $file
            ),
            mirror_urls: &[concat!(
                "https://github.com/redis-windows/redis-windows/releases/download/",
                $tag,
                "/",
                $file
            )],
            engine: None,
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub default_port: u16,
    pub default_version_id: &'static str,
    pub versions: &'static [VersionEntry],
}

pub const WINDOWS_COMPONENTS: &[ComponentManifest] = &[
    ComponentManifest {
        id: "mysql",
        name: "MySQL",
        default_port: 3307,
        default_version_id: "mysql-8.4.4",
        versions: &[
            VersionEntry {
                id: "mysql-8.4.4",
                label: "MySQL 8.4.4",
                filename: "mysql-8.4.4-winx64.zip",
                url: "https://cdn.mysql.com/Downloads/MySQL-8.4/mysql-8.4.4-winx64.zip",
                mirror_urls: &[],
                engine: Some(DbEngine::Mysql),
            },
            VersionEntry {
                id: "mysql-8.4.3",
                label: "MySQL 8.4.3",
                filename: "mysql-8.4.3-winx64.zip",
                url: "https://cdn.mysql.com/Downloads/MySQL-8.4/mysql-8.4.3-winx64.zip",
                mirror_urls: &[],
                engine: Some(DbEngine::Mysql),
            },
            VersionEntry {
                id: "mysql-8.0.40",
                label: "MySQL 8.0.40",
                filename: "mysql-8.0.40-winx64.zip",
                url: "https://cdn.mysql.com/Downloads/MySQL-8.0/mysql-8.0.40-winx64.zip",
                mirror_urls: &[],
                engine: Some(DbEngine::Mysql),
            },
            VersionEntry {
                id: "mariadb-11.4.2",
                label: "MariaDB 11.4.2",
                filename: "mariadb-11.4.2-winx64.zip",
                url: "https://archive.mariadb.org/mariadb-11.4.2/winx64-packages/mariadb-11.4.2-winx64.zip",
                mirror_urls: &[],
                engine: Some(DbEngine::MariaDb),
            },
        ],
    },
    ComponentManifest {
        id: "nginx",
        name: "Nginx",
        default_port: 8080,
        default_version_id: "1.26.3",
        versions: &[
            VersionEntry {
                id: "1.26.3",
                label: "Nginx 1.26.3",
                filename: "nginx-1.26.3.zip",
                url: "https://nginx.org/download/nginx-1.26.3.zip",
                mirror_urls: &[],
                engine: None,
            },
            VersionEntry {
                id: "1.25.5",
                label: "Nginx 1.25.5",
                filename: "nginx-1.25.5.zip",
                url: "https://nginx.org/download/nginx-1.25.5.zip",
                mirror_urls: &[],
                engine: None,
            },
            VersionEntry {
                id: "1.24.0",
                label: "Nginx 1.24.0",
                filename: "nginx-1.24.0.zip",
                url: "https://nginx.org/download/nginx-1.24.0.zip",
                mirror_urls: &[],
                engine: None,
            },
        ],
    },
    ComponentManifest {
        id: "php",
        name: "PHP",
        default_port: 9000,
        default_version_id: "8.3.31",
        versions: &[
            php_entry!("8.3.31", "PHP 8.3.31 (NTS)", "8.3.31", "vs16"),
            php_entry!("8.4.22", "PHP 8.4.22 (NTS)", "8.4.22", "vs17"),
            php_entry!("8.2.29", "PHP 8.2.29 (NTS)", "8.2.29", "vs16"),
        ],
    },
    ComponentManifest {
        id: "redis",
        name: "Redis",
        default_port: 6379,
        default_version_id: "7.4.9",
        versions: &[
            redis_entry!(
                "7.4.9",
                "Redis 7.4.9",
                "7.4.9",
                "Redis-7.4.9-Windows-x64-msys2.zip"
            ),
            redis_entry!(
                "7.4.8",
                "Redis 7.4.8",
                "7.4.8",
                "Redis-7.4.8-Windows-x64-msys2.zip"
            ),
            redis_entry!(
                "7.2.14",
                "Redis 7.2.14",
                "7.2.14",
                "Redis-7.2.14-Windows-x64-msys2.zip"
            ),
            redis_entry!(
                "7.0.15",
                "Redis 7.0.15",
                "7.0.15",
                "Redis-7.0.15-Windows-x64-msys2.zip"
            ),
            VersionEntry {
                id: "5.0.14.1",
                label: "Redis 5.0.14.1（旧版）",
                filename: "Redis-x64-5.0.14.1.zip",
                url: "https://ghfast.top/https://github.com/tporadowski/redis/releases/download/v5.0.14.1/Redis-x64-5.0.14.1.zip",
                mirror_urls: &["https://github.com/tporadowski/redis/releases/download/v5.0.14.1/Redis-x64-5.0.14.1.zip"],
                engine: None,
            },
        ],
    },
];

pub fn get_component(id: &str) -> Result<&'static ComponentManifest, String> {
    WINDOWS_COMPONENTS
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("未知组件: {id}"))
}

pub fn find_version(component_id: &str, version_id: &str) -> Result<&'static VersionEntry, String> {
    let comp = get_component(component_id)?;
    comp.versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("未知版本: {component_id}@{version_id}"))
}

pub fn resolve_version(
    component_id: &str,
    version_id: Option<&str>,
) -> Result<(&'static ComponentManifest, &'static VersionEntry), String> {
    let comp = get_component(component_id)?;
    let id = version_id.unwrap_or(comp.default_version_id);
    let ver = find_version(component_id, id)?;
    Ok((comp, ver))
}

pub fn default_version_id(component_id: &str) -> Result<&'static str, String> {
    Ok(get_component(component_id)?.default_version_id)
}

/// 兼容旧接口
pub fn get(id: &str) -> Result<LegacyManifest, String> {
    let (comp, ver) = resolve_version(id, None)?;
    Ok(LegacyManifest {
        id: comp.id,
        name: comp.name,
        version: ver.label,
        filename: ver.filename,
        url: ver.url,
        default_port: comp.default_port,
    })
}

pub struct LegacyManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    pub default_port: u16,
}
