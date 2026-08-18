pub mod discovery;
pub mod execute;
pub mod install;
pub mod registry;
pub mod search;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub scope: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub repository: String,
    pub commands: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub name: String,
    pub scope: String,
    pub version: String,
    pub installed: bool,
    pub commands: Vec<String>,
    pub last_checked: String,
}

pub fn plugins_dir() -> PathBuf {
    crate::utils::config_dir().join("plugins")
}

pub fn registry_file() -> PathBuf {
    crate::utils::config_dir().join("registry.toml")
}

pub fn plugin_dir(scope: &str, name: &str) -> PathBuf {
    plugins_dir().join(format!("{}-{}", scope, name))
}

pub fn parse_plugin_ref(reference: &str) -> (String, String) {
    if let Some(without_at) = reference.strip_prefix('@') {
        if let Some(slash_pos) = without_at.find('/') {
            let scope = without_at[..slash_pos].to_string();
            let name = without_at[slash_pos + 1..].to_string();
            (scope, name)
        } else {
            ("proto".to_string(), without_at.to_string())
        }
    } else {
        ("proto".to_string(), reference.to_string())
    }
}
