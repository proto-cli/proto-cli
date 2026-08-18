use super::registry;
use super::{plugin_dir, InstalledPlugin, PluginInfo, PluginManifest};
use crate::globals;
use crate::style;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fs;
use std::io::Read;
use std::path::Path;

const GITHUB_API_BASE: &str = "https://api.github.com/repos/proto-cli/plugins";
const GITHUB_REPO_BASE: &str = "https://github.com/proto-cli/plugins";

pub fn install_plugin(reference: &str) -> Result<(), String> {
    let (scope, name) = super::parse_plugin_ref(reference);

    let installed_dir = plugin_dir(&scope, &name);
    if installed_dir.exists() {
        if !globals::is_quiet() {
            println!(
                "{}",
                format!("Plugin {} is already installed", reference).style(style::Theme::SUCCESS)
            );
        }
        return Ok(());
    }

    if !globals::is_quiet() {
        println!(
            "{}",
            format!("Installing {}...", reference).style(style::Theme::HEADER)
        );
    }

    let release = fetch_latest_release(&name)?;

    if let Some(release) = release {
        if !globals::is_quiet() {
            println!(
                "{}",
                format!("Found release {}", release.tag_name).style(style::Theme::MUTED)
            );
        }
        download_release_binary(&name, &release, &installed_dir)?;
    } else {
        if !globals::is_quiet() {
            println!(
                "{}",
                "No pre-compiled release found.".style(style::Theme::WARN)
            );
        }
        if crate::utils::which("cargo") {
            if !globals::is_quiet() {
                println!("{}", "Compiling from source...".style(style::Theme::MUTED));
            }
            compile_from_source(&name, &installed_dir)?;
        } else {
            return Err(format!(
                "No release available and cargo not found. Cannot install {}.",
                reference
            ));
        }
    }

    let manifest_path = installed_dir.join("plugin.toml");
    if !manifest_path.exists() {
        create_manifest(&name, &scope, &installed_dir)?;
    }

    let info = read_manifest(&installed_dir)?;
    let plugin = InstalledPlugin {
        name: info.name.clone(),
        scope: scope.clone(),
        version: info.version.clone(),
        installed: true,
        commands: info.commands.keys().cloned().collect(),
        last_checked: chrono_now(),
    };

    registry::add_plugin(plugin)?;

    if !globals::is_quiet() {
        println!(
            "{}",
            format!("Successfully installed {} v{}", reference, info.version)
                .style(style::Theme::SUCCESS)
        );
        println!(
            "{}",
            format!(
                "Run 'proto {} --help' to get started",
                info.commands.keys().next().unwrap_or(&info.name)
            )
            .style(style::Theme::MUTED)
        );
    }

    Ok(())
}

pub fn remove_plugin(reference: &str) -> Result<(), String> {
    let (scope, name) = super::parse_plugin_ref(reference);
    let installed_dir = plugin_dir(&scope, &name);

    if !installed_dir.exists() {
        return Err(format!("Plugin {} is not installed", reference));
    }

    if !globals::is_quiet() {
        let prompt = format!("Remove plugin {}?", reference);
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(&prompt)
            .default(false)
            .interact()
            .map_err(|e| format!("Failed to get confirmation: {}", e))?;

        if !confirmed {
            return Ok(());
        }
    }

    if !globals::is_quiet() {
        println!(
            "{}",
            format!("Removing {}...", reference).style(style::Theme::HEADER)
        );
    }

    fs::remove_dir_all(&installed_dir)
        .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;

    registry::remove_plugin(&name)?;

    if !globals::is_quiet() {
        println!(
            "{}",
            format!("Successfully removed {}", reference).style(style::Theme::SUCCESS)
        );
    }
    Ok(())
}

pub fn update_plugin(reference: Option<&str>) -> Result<(), String> {
    if let Some(name) = reference {
        update_single_plugin(name)
    } else {
        update_all_plugins()
    }
}

fn update_single_plugin(name: &str) -> Result<(), String> {
    let plugin =
        registry::get_plugin(name).ok_or_else(|| format!("Plugin {} is not installed", name))?;

    println!(
        "{}",
        format!("Checking for updates to {}...", name).style(style::Theme::HEADER)
    );

    let release = fetch_latest_release(name)?;
    if let Some(release) = release {
        let latest_version = release.tag_name.trim_start_matches('v');
        if latest_version == plugin.version {
            println!(
                "{}",
                format!("{} is already up to date (v{})", name, plugin.version)
                    .style(style::Theme::SUCCESS)
            );
            return Ok(());
        }

        println!(
            "{}",
            format!(
                "Updating {} from v{} to v{}",
                name, plugin.version, latest_version
            )
            .style(style::Theme::MUTED)
        );

        let installed_dir = plugin_dir(&plugin.scope, name);
        fs::remove_dir_all(&installed_dir)
            .map_err(|e| format!("Failed to remove old version: {}", e))?;

        download_release_binary(name, &release, &installed_dir)?;

        let info = read_manifest(&installed_dir)?;
        let updated = InstalledPlugin {
            name: info.name.clone(),
            scope: plugin.scope.clone(),
            version: info.version.clone(),
            installed: true,
            commands: info.commands.keys().cloned().collect(),
            last_checked: chrono_now(),
        };
        registry::add_plugin(updated)?;

        println!(
            "{}",
            format!("Updated {} to v{}", name, info.version).style(style::Theme::SUCCESS)
        );
    } else {
        println!(
            "{}",
            format!("No release found for {}", name).style(style::Theme::WARN)
        );
    }
    Ok(())
}

fn update_all_plugins() -> Result<(), String> {
    let plugins = registry::list_plugins();
    let installed: Vec<_> = plugins.into_iter().filter(|p| p.installed).collect();

    if installed.is_empty() {
        println!("{}", "No plugins installed.".style(style::Theme::MUTED));
        return Ok(());
    }

    for plugin in installed {
        if let Err(e) = update_single_plugin(&plugin.name) {
            println!(
                "{}",
                format!("Failed to update {}: {}", plugin.name, e).style(style::Theme::WARN)
            );
        }
    }
    Ok(())
}

struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

fn fetch_latest_release(plugin_name: &str) -> Result<Option<GitHubRelease>, String> {
    let config = crate::utils::load_config();
    let mut urls = vec![format!("{}/releases/latest", GITHUB_API_BASE)];

    if let Some(custom_repos) = config.custom_repos {
        for repo in custom_repos {
            let api_url = repo
                .replace("https://github.com/", "https://api.github.com/repos/")
                .replace("http://github.com/", "https://api.github.com/repos/")
                .replace("github.com/", "https://api.github.com/repos/");
            if !api_url.contains("/releases/latest") {
                urls.push(format!("{}/releases/latest", api_url));
            }
        }
    }

    let agent = ureq::Agent::new();

    for url in &urls {
        let response = agent.get(url).set("User-Agent", "proto-cli").call();

        match response {
            Ok(resp) => {
                let body: serde_json::Value = resp.into_json().unwrap_or_default();
                let tag = body["tag_name"].as_str().unwrap_or("").to_string();
                let assets: Vec<GitHubAsset> = body["assets"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .map(|a| GitHubAsset {
                                name: a["name"].as_str().unwrap_or("").to_string(),
                                browser_download_url: a["browser_download_url"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                if tag.is_empty() {
                    continue;
                }

                let binary_name = get_asset_name_for_platform(plugin_name);
                let has_binary = assets.iter().any(|a| a.name.contains(&binary_name));

                if has_binary {
                    return Ok(Some(GitHubRelease {
                        tag_name: tag,
                        assets,
                    }));
                }
            }
            Err(ureq::Error::Status(404, _)) => continue,
            Err(_) => continue,
        }
    }

    Ok(None)
}

fn download_release_binary(
    plugin_name: &str,
    release: &GitHubRelease,
    install_dir: &Path,
) -> Result<(), String> {
    let binary_name = get_asset_name_for_platform(plugin_name);
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(&binary_name))
        .ok_or_else(|| format!("Binary {} not found in release", binary_name))?;

    fs::create_dir_all(install_dir.join("bin"))
        .map_err(|e| format!("Failed to create bin directory: {}", e))?;

    let binary_path = install_dir.join("bin").join(plugin_name);

    if !globals::is_quiet() {
        println!(
            "{}",
            format!("Downloading {}...", asset.name).style(style::Theme::MUTED)
        );
    }

    let agent = ureq::Agent::new();
    let response = agent
        .get(&asset.browser_download_url)
        .set("User-Agent", "proto-cli")
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let total_size = response
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut reader = pb.wrap_read(response.into_reader());
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read download: {}", e))?;
    pb.finish_with_message("Downloaded");

    fs::write(&binary_path, bytes).map_err(|e| format!("Failed to write binary: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .map_err(|e| format!("Failed to read permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

fn compile_from_source(plugin_name: &str, install_dir: &Path) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join(format!("proto-plugin-{}", plugin_name));

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).ok();
    }

    let clone_url = format!("{}.git", GITHUB_REPO_BASE);

    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &clone_url,
            &temp_dir.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Failed to clone plugin source: {}", e))?;

    if !status.success() {
        return Err(format!("Failed to clone plugin source for {}", plugin_name));
    }

    let build_status = std::process::Command::new("cargo")
        .args(["build", "--release", "-p", plugin_name])
        .current_dir(&temp_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Failed to build plugin: {}", e))?;

    if !build_status.success() {
        return Err(format!("Failed to compile plugin {}", plugin_name));
    }

    fs::create_dir_all(install_dir.join("bin"))
        .map_err(|e| format!("Failed to create bin directory: {}", e))?;

    let binary_source = temp_dir.join("target").join("release").join(plugin_name);
    let binary_dest = install_dir.join("bin").join(plugin_name);

    fs::copy(&binary_source, &binary_dest)
        .map_err(|e| format!("Failed to copy compiled binary: {}", e))?;

    fs::remove_dir_all(&temp_dir).ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_dest)
            .map_err(|e| format!("Failed to read permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_dest, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

fn create_manifest(name: &str, scope: &str, dir: &Path) -> Result<(), String> {
    let manifest = PluginManifest {
        plugin: PluginInfo {
            name: name.to_string(),
            scope: scope.to_string(),
            version: "0.1.0".to_string(),
            description: format!("{} plugin for proto", name),
            author: "proto-cli".to_string(),
            repository: format!("{}/tree/main/{}", GITHUB_REPO_BASE, name),
            commands: {
                let mut m = std::collections::HashMap::new();
                m.insert(name.to_string(), format!("bin/{}", name));
                m
            },
        },
    };

    let toml_str = toml::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(dir.join("plugin.toml"), toml_str)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

fn read_manifest(dir: &Path) -> Result<PluginInfo, String> {
    let content = fs::read_to_string(dir.join("plugin.toml"))
        .map_err(|e| format!("Failed to read plugin manifest: {}", e))?;
    let manifest: PluginManifest =
        toml::from_str(&content).map_err(|e| format!("Failed to parse plugin manifest: {}", e))?;
    Ok(manifest.plugin)
}

fn get_asset_name_for_platform(plugin_name: &str) -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let platform = match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => {
            if os == "windows" {
                "x86_64-pc-windows-msvc"
            } else {
                "x86_64-unknown-linux-gnu"
            }
        }
    };

    if os == "windows" {
        format!("{}-{}.exe", plugin_name, platform)
    } else {
        format!("{}-{}", plugin_name, platform)
    }
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
