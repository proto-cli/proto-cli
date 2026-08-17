use super::{InstalledPlugin, registry_file};
use std::fs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginRegistry {
    pub plugins: Vec<InstalledPlugin>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
}

pub fn load_registry() -> PluginRegistry {
    let path = registry_file();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        PluginRegistry::default()
    }
}

pub fn save_registry(registry: &PluginRegistry) -> Result<(), String> {
    let dir = registry_file().parent().unwrap().to_path_buf();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let toml_str = toml::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;
    fs::write(registry_file(), toml_str)
        .map_err(|e| format!("Failed to write registry: {}", e))?;
    Ok(())
}

pub fn add_plugin(plugin: InstalledPlugin) -> Result<(), String> {
    let mut registry = load_registry();
    if let Some(existing) = registry.plugins.iter_mut().find(|p| p.name == plugin.name) {
        existing.version = plugin.version;
        existing.installed = plugin.installed;
        existing.commands = plugin.commands;
        existing.last_checked = plugin.last_checked;
    } else {
        registry.plugins.push(plugin);
    }
    save_registry(&registry)
}

pub fn remove_plugin(name: &str) -> Result<(), String> {
    let mut registry = load_registry();
    registry.plugins.retain(|p| p.name != name);
    save_registry(&registry)
}

pub fn get_plugin(name: &str) -> Option<InstalledPlugin> {
    let registry = load_registry();
    registry.plugins.into_iter().find(|p| p.name == name)
}

pub fn list_plugins() -> Vec<InstalledPlugin> {
    load_registry().plugins
}

#[allow(dead_code)]
pub fn find_plugin_for_command(command: &str) -> Option<InstalledPlugin> {
    let registry = load_registry();
    registry
        .plugins
        .into_iter()
        .filter(|p| p.installed)
        .find(|p| p.commands.contains(&command.to_string()))
}
