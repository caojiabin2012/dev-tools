use std::path::Path;

use crate::stack::store::require_install_root;
use crate::stack::types::StackStore;

pub fn sync_all_configs() -> Result<(), String> {
    let store = crate::stack::store::load_store();
    let root = require_install_root()?;
    sync_configs(&store, &root)
}

pub fn sync_configs(store: &StackStore, install_root: &Path) -> Result<(), String> {
    if let Some(mysql) = &store.mysql {
        crate::stack::mysql::config::write_config(install_root, mysql)?;
    }
    if let Some(php) = &store.php {
        crate::stack::php::write_config(install_root, php)?;
    }
    if let Some(redis) = &store.redis {
        crate::stack::redis::write_config(install_root, redis)?;
    }
    if let Some(nginx) = &store.nginx {
        let php_port = store.php.as_ref().map(|p| p.port).unwrap_or(9000);
        let www = crate::stack::store::resolve_www_root(install_root, &store.settings);
        crate::stack::nginx::write_config(nginx, php_port, &www)?;
    }
    crate::stack::www::sync_site_files(store, install_root)?;
    Ok(())
}
