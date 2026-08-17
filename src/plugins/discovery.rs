use super::{PluginInfo, PluginManifest, plugin_dir, plugins_dir};
use std::fs;
use std::path::PathBuf;

pub fn scan_installed_plugins() -> Vec<(PluginInfo, PathBuf)> {
    let plugins_root = plugins_dir();
    if !plugins_root.exists() {
        return Vec::new();
    }

    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(&plugins_root) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let manifest_path = entry.path().join("plugin.toml");
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = toml::from_str::<PluginManifest>(&content) {
                        let info = manifest.plugin;
                        let binary_path = resolve_binary_path(&entry.path(), &info);
                        if binary_path.exists() {
                            result.push((info, binary_path));
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn find_plugin_binary(scope: &str, name: &str) -> Option<PathBuf> {
    let dir = plugin_dir(scope, name);
    if !dir.exists() {
        return None;
    }
    let manifest_path = dir.join("plugin.toml");
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = toml::from_str::<PluginManifest>(&content) {
            let binary = resolve_binary_path(&dir, &manifest.plugin);
            if binary.exists() {
                return Some(binary);
            }
        }
    }
    None
}

pub fn find_plugin_for_command(command: &str) -> Option<(PluginInfo, PathBuf)> {
    let plugins = scan_installed_plugins();
    for (info, binary) in plugins {
        if info.commands.contains_key(command) {
            return Some((info, binary));
        }
    }
    None
}

fn resolve_binary_path(plugin_dir: &PathBuf, info: &PluginInfo) -> PathBuf {
    if let Some((_, rel_path)) = info.commands.iter().next() {
        let binary_name = PathBuf::from(rel_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| info.name.clone());
        plugin_dir.join("bin").join(binary_name)
    } else {
        plugin_dir.join("bin").join(&info.name)
    }
}

#[allow(dead_code)]
pub fn get_plugin_info(scope: &str, name: &str) -> Option<PluginInfo> {
    let dir = plugin_dir(scope, name);
    let manifest_path = dir.join("plugin.toml");
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        toml::from_str::<PluginManifest>(&content).ok().map(|m| m.plugin)
    } else {
        None
    }
}
