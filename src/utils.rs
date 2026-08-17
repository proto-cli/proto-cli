use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtoConfig {
    pub default_pm: Option<String>,
    pub install_dir: Option<String>,
    pub color: Option<bool>,
    pub completions_installed: Option<bool>,
    pub custom_repos: Option<Vec<String>>,
}

impl Default for ProtoConfig {
    fn default() -> Self {
        Self {
            default_pm: None,
            install_dir: None,
            color: Some(true),
            completions_installed: Some(false),
            custom_repos: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config").into())
        .join("proto")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> ProtoConfig {
    let path = config_file();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        ProtoConfig::default()
    }
}

pub fn save_config(config: &ProtoConfig) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    let toml_str = toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(config_file(), toml_str).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Pacman,
    Yay,
    Paru,
    Apt,
    Dnf,
    Zypper,
    Apk,
    Unknown,
}

impl PackageManager {
    pub fn name(&self) -> &str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Yay => "yay",
            PackageManager::Paru => "paru",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
            PackageManager::Unknown => "unknown",
        }
    }

    pub fn is_aur_helper(&self) -> bool {
        matches!(self, PackageManager::Yay | PackageManager::Paru)
    }
}

pub fn detect_package_managers() -> Vec<PackageManager> {
    let mut managers = Vec::new();

    let checks: &[(&str, PackageManager)] = &[
        ("pacman", PackageManager::Pacman),
        ("yay", PackageManager::Yay),
        ("paru", PackageManager::Paru),
        ("apt", PackageManager::Apt),
        ("dnf", PackageManager::Dnf),
        ("zypper", PackageManager::Zypper),
        ("apk", PackageManager::Apk),
    ];

    for (bin, pm) in checks {
        if which(bin) {
            managers.push(pm.clone());
        }
    }

    if managers.is_empty() {
        managers.push(PackageManager::Unknown);
    }

    managers
}

pub fn default_package_manager() -> PackageManager {
    let managers = detect_package_managers();
    for pm in &managers {
        if pm.is_aur_helper() {
            return pm.clone();
        }
    }
    managers.into_iter().next().unwrap_or(PackageManager::Unknown)
}
pub fn which(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn run_command(program: &str, args: &[&str]) -> std::io::Result<std::process::ExitStatus> {
    Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
}

pub fn run_command_output(program: &str, args: &[&str]) -> std::io::Result<String> {
    let output = Command::new(program).args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn distro_name() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                return name.trim_matches('"').to_string();
            }
        }
    }
    "Unknown".to_string()
}

pub fn get_package_count(pm: &PackageManager) -> Option<usize> {
    let (cmd, args): (&str, &[&str]) = match pm {
        PackageManager::Pacman | PackageManager::Yay | PackageManager::Paru => {
            ("pacman", &["-Q"] as &[&str])
        }
        PackageManager::Apt => ("dpkg-query", &["-f", "${Package}\n", "-W"]),
        PackageManager::Dnf => ("rpm", &["-qa"]),
        PackageManager::Zypper => ("rpm", &["-qa"]),
        PackageManager::Apk => ("apk", &["info"]),
        PackageManager::Unknown => return None,
    };

    run_command_output(cmd, args)
        .ok()
        .map(|s| s.lines().count())
}

pub fn get_uptime() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs_str) = content.split_whitespace().next() {
            if let Ok(total_secs) = secs_str.parse::<f64>() {
                let total_secs = total_secs as u64;
                let days = total_secs / 86400;
                let hours = (total_secs % 86400) / 3600;
                let minutes = (total_secs % 3600) / 60;

                let mut parts = Vec::new();
                if days > 0 { parts.push(format!("{}d", days)); }
                if hours > 0 { parts.push(format!("{}h", hours)); }
                if minutes > 0 { parts.push(format!("{}m", minutes)); }
                if parts.is_empty() { parts.push("just now".to_string()); }
                return parts.join(" ");
            }
        }
    }
    "Unknown".to_string()
}

pub fn get_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "Unknown".to_string())
        .split('/')
        .last()
        .unwrap_or("Unknown")
        .to_string()
}

pub fn get_de_wm() -> String {
    let envs = [
        ("XDG_CURRENT_DESKTOP", ""),
        ("DESKTOP_SESSION", ""),
        ("XDG_SESSION_DESKTOP", ""),
    ];

    for (var, _) in &envs {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                return val.to_lowercase();
            }
        }
    }
    "tty".to_string()
}

pub fn get_terminal() -> String {
    std::env::var("TERM")
        .unwrap_or_else(|_| "Unknown".to_string())
}

pub fn get_kernel() -> String {
    run_command_output("uname", &["-r"]).unwrap_or_else(|_| "Unknown".to_string())
}

pub fn get_arch() -> String {
    run_command_output("uname", &["-m"]).unwrap_or_else(|_| "Unknown".to_string())
}
