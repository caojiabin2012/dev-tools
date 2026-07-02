use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine {
    Mysql,
    MariaDb,
}

impl DbEngine {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mysql => "MySQL",
            Self::MariaDb => "MariaDB",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    NotInstalled,
    Stopped,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlInstall {
    pub engine: DbEngine,
    pub version_label: String,
    pub home_dir: String,
    pub port: u16,
    pub initialized: bool,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NginxInstall {
    pub version_label: String,
    pub home_dir: String,
    pub port: u16,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpInstall {
    pub version_label: String,
    pub home_dir: String,
    pub port: u16,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisInstall {
    pub version_label: String,
    pub home_dir: String,
    pub port: u16,
    #[serde(default)]
    pub pid: Option<u32>,
}

fn default_www_subdir() -> String {
    "www/default".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackSettings {
    /// 相对 install_root 的网站目录
    #[serde(default = "default_www_subdir")]
    pub www_subdir: String,
}

impl Default for StackSettings {
    fn default() -> Self {
        Self {
            www_subdir: default_www_subdir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StackStore {
    pub install_root: Option<String>,
    pub mysql: Option<MysqlInstall>,
    pub nginx: Option<NginxInstall>,
    pub php: Option<PhpInstall>,
    pub redis: Option<RedisInstall>,
    /// 组件 id -> 版本 id
    #[serde(default)]
    pub version_prefs: HashMap<String, String>,
    #[serde(default)]
    pub settings: StackSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StackEnvInfo {
    pub site_url: Option<String>,
    pub www_root: Option<String>,
    pub mysql_host: String,
    pub mysql_port: Option<u16>,
    pub mysql_user: String,
    pub mysql_password: String,
    pub php_fastcgi: Option<String>,
    pub redis_addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionOption {
    pub id: String,
    pub label: String,
    pub engine: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentView {
    pub id: String,
    pub name: String,
    /// 当前选中的版本 id
    pub selected_version_id: String,
    /// 当前选中的版本显示名
    pub selected_version_label: String,
    pub available_versions: Vec<VersionOption>,
    pub default_port: u16,
    pub downloaded: bool,
    pub download_path: Option<String>,
    pub installed: bool,
    pub status: ServiceStatus,
    pub port: Option<u16>,
    pub home_dir: Option<String>,
    pub pid: Option<u32>,
    pub hint: Option<String>,
    pub config_path: Option<String>,
    pub log_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackState {
    pub install_root: Option<String>,
    pub settings: StackSettings,
    pub env_info: StackEnvInfo,
    pub components: Vec<ComponentView>,
}

#[derive(Debug, Deserialize)]
pub struct InstallComponentParams {
    pub component: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub engine: Option<DbEngine>,
}

#[derive(Debug, Deserialize)]
pub struct SetComponentVersionParams {
    pub component: String,
    pub version_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStackSettingsParams {
    pub www_subdir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetComponentPortParams {
    pub component: String,
    pub port: u16,
}
