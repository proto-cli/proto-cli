use crate::style;
use owo_colors::OwoColorize;
use std::time::{Duration, SystemTime};

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/proto-cli/proto-cli/releases/latest";
const CHECK_INTERVAL: Duration = Duration::from_secs(3600 * 24);

fn version_file() -> std::path::PathBuf {
    crate::utils::config_dir().join("last_update_check")
}

fn read_last_check() -> Option<SystemTime> {
    let content = std::fs::read_to_string(version_file()).ok()?;
    let secs: u64 = content.trim().parse().ok()?;
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(secs))
}

fn write_last_check() {
    let dir = crate::utils::config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let _ = std::fs::write(version_file(), now.to_string());
}

fn compare_versions(current: &str, latest: &str) -> Option<bool> {
    let cur: Vec<u32> = current
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let lat: Vec<u32> = latest
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if cur.len() < 3 || lat.len() < 3 {
        return None;
    }

    if lat[0] > cur[0] {
        return Some(true);
    }
    if lat[0] == cur[0] && lat[1] > cur[1] {
        return Some(true);
    }
    if lat[0] == cur[0] && lat[1] == cur[1] && lat[2] > cur[2] {
        return Some(true);
    }
    Some(false)
}

fn fetch_latest_version() -> Option<String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .user_agent("proto-cli")
        .build();

    let resp = agent.get(GITHUB_API_LATEST).call().ok()?;
    let body: serde_json::Value = resp.into_json().ok()?;
    let tag = body["tag_name"].as_str()?;
    Some(tag.to_string())
}

pub fn check_for_update() {
    if crate::globals::is_quiet() {
        return;
    }

    if let Some(last) = read_last_check() {
        if let Ok(elapsed) = SystemTime::now().duration_since(last) {
            if elapsed < CHECK_INTERVAL {
                return;
            }
        }
    }

    write_last_check();

    let current = env!("CARGO_PKG_VERSION");
    if let Some(latest) = fetch_latest_version() {
        if let Some(is_outdated) = compare_versions(current, &latest) {
            if is_outdated {
                eprintln!(
                    "\n{} {}\n",
                    "⚠ A new version of proto is available:".style(style::Theme::WARN),
                    format!("{} -> {}", current, latest).style(style::Theme::ACCENT)
                );
                eprintln!(
                    "  {}\n",
                    "Run 'proto manage update' to upgrade.".style(style::Theme::MUTED)
                );
            }
        }
    }
}
