use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum ManageAction {
    #[command(about = "Pull latest source and rebuild Proto CLI")]
    Update,
    #[command(about = "Remove Proto CLI from your system")]
    Uninstall {
        #[arg(long, help = "Also delete the cloned repository")]
        purge: bool,
    },
    #[command(about = "Reset Proto configuration and state")]
    Reset,
}

pub fn run(action: &ManageAction) {
    match action {
        ManageAction::Update => update(),
        ManageAction::Uninstall { purge } => uninstall(*purge),
        ManageAction::Reset => reset(),
    }
}

fn repo_dir() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

fn detect_target() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc.exe",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc.exe",
        _ => "",
    };
    if target.is_empty() {
        eprintln!(
            "  {} No prebuilt binary for {}-{}",
            style::error(""),
            os,
            arch
        );
    }
    target.to_string()
}

fn latest_tag() -> Result<String, String> {
    let agent = ureq::Agent::new();
    agent
        .get("https://api.github.com/repos/proto-cli/proto-cli/releases/latest")
        .set("User-Agent", "proto-cli")
        .call()
        .map_err(|e| format!("Could not reach GitHub releases: {}", e))
        .and_then(|resp| {
            let body: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
            body["tag_name"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "No tag_name in latest release".into())
        })
}

fn download_to(
    url: &str,
    dest: &std::path::Path,
) -> Result<(), String> {
    let agent = ureq::Agent::new();
    let response = agent
        .get(url)
        .set("User-Agent", "proto-cli")
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let total_size = response
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = indicatif::ProgressBar::new(total_size);
    pb.set_style(
        indicatif::ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut reader = pb.wrap_read(response.into_reader());
    let mut bytes = Vec::new();
    use std::io::Read;
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Download error: {}", e))?;
    pb.finish_and_clear();

    std::fs::write(dest, &bytes).map_err(|e| format!("Write error: {}", e))
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    digest
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn update() {
    println!("{}", style::header("Proto Update"));
    println!("{}", style::divider());

    let current = std::env::current_exe().unwrap_or_default();
    let target = detect_target();
    if target.is_empty() {
        return;
    }

    let spin = style::Spinner::new("Checking for latest release...");
    let tag = match latest_tag() {
        Ok(t) => t,
        Err(e) => {
            spin.fail(&e);
            return;
        }
    };
    spin.done(&format!("Latest: {}", tag));

    let exe_suffix = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let asset_name = format!("proto-{}{}", target.trim_end_matches(".exe"), exe_suffix);
    let base = format!(
        "https://github.com/proto-cli/proto-cli/releases/download/{}/{}",
        tag, asset_name
    );

    // Stage into a temp dir next to the current executable so the final
    // swap can be an atomic rename (avoids ETXTBSY on the live binary).
    let tmp_dir = std::env::temp_dir().join(format!("proto-update-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).ok();
    let tmp_bin = tmp_dir.join("proto");

    println!("  {} Downloading {}", style::muted(""), &asset_name);
    if let Err(e) = download_to(&base, &tmp_bin) {
        eprintln!("  {} {}", style::error(""), e);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }

    // Verify checksum when a published .sha256 exists.
    let sha_url = format!("{}.sha256", base);
    let sha_text = fetch_plain(&sha_url);
    match sha_text {
        Some(text) => {
            let hex = text
                .split_whitespace()
                .next()
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            if hex.is_empty() {
                println!("  {} Could not parse checksum file", style::warn(""));
            } else {
                let actual = std::fs::read(&tmp_bin)
                    .map(|b| sha256_hex(&b))
                    .unwrap_or_default();
                if actual == hex {
                    println!("  {} Checksum verified", style::success(""));
                } else {
                    eprintln!("  {} Checksum mismatch:", style::error(""));
                    eprintln!("    expected: {}", hex);
                    eprintln!("    actual:   {}", actual);
                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    return;
                }
            }
        }
        None => println!(
            "  {} No published checksum to verify against",
            style::warn("")
        ),
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            &tmp_bin,
            std::fs::Permissions::from_mode(0o755),
        );
    }

    // Atomic swap: copy to sibling temp name, then rename over the live binary.
    let mut tmp_name = current
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp = current.with_file_name(tmp_name);

    if std::fs::copy(&tmp_bin, &tmp)
        .and_then(|_| std::fs::rename(&tmp, &current))
        .is_err()
    {
        let _ = std::fs::remove_file(&tmp);
        match std::fs::remove_file(&current).and_then(|_| std::fs::copy(&tmp_bin, &current)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("  {} Failed to install: {}", style::error(""), e);
                let _ = std::fs::remove_dir_all(&tmp_dir);
                return;
            }
        }
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("  {} Updated to {}.", style::success(""), tag);
    println!(
        "  {} Run `proto --version` to confirm.",
        style::muted("")
    );
}

fn fetch_plain(url: &str) -> Option<String> {
    let agent = ureq::Agent::new();
    match agent.get(url).set("User-Agent", "proto-cli").call() {
        Ok(response) => {
            use std::io::Read;
            let mut bytes = Vec::new();
            response
                .into_reader()
                .read_to_end(&mut bytes)
                .ok()?;
            Some(String::from_utf8_lossy(&bytes).to_string())
        }
        Err(ureq::Error::Status(404, _)) => None,
        Err(_) => None,
    }
}

fn uninstall(purge: bool) {
    println!("{}", style::header("Proto Uninstall"));
    println!("{}", style::divider());

    let current = std::env::current_exe().unwrap_or_default();
    println!(
        "  Binary: {}",
        current.display().to_string().style(style::Theme::VALUE)
    );

    let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Remove the proto binary?")
        .default(false)
        .interact()
        .unwrap_or(false);

    if confirm {
        if let Err(e) = std::fs::remove_file(&current) {
            eprintln!("  {} Failed to remove binary: {}", style::error(""), e);
        } else {
            println!("  {} Binary removed.", style::success(""));
        }
    } else {
        println!("  {} Cancelled.", style::muted(""));
        return;
    }

    if purge {
        let repo = repo_dir();
        println!("\n  {} Purging repository at {}", style::warn(""), repo);
        let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Remove the entire proto repository?")
            .default(false)
            .interact()
            .unwrap_or(false);
        if confirm {
            if let Err(e) = std::fs::remove_dir_all(repo) {
                eprintln!("  {} Failed to remove repo: {}", style::error(""), e);
            } else {
                println!("  {} Repository removed.", style::success(""));
            }
        } else {
            println!("  {} Repo kept.", style::muted(""));
        }
    }
}

fn reset() {
    println!("{}", style::header("Proto Reset"));
    println!("{}", style::divider());

    let dirs = vec![
        dirs::config_dir().map(|d| d.join("proto")),
        dirs::data_local_dir().map(|d| d.join("proto")),
        dirs::home_dir().map(|d| d.join(".proto")),
    ];

    let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Remove all proto config and state directories?")
        .default(false)
        .interact()
        .unwrap_or(false);

    if !confirm {
        println!("  {} Cancelled.", style::muted(""));
        return;
    }

    for dir in dirs.into_iter().flatten() {
        if dir.exists() {
            println!("  {} Removing {}...", style::muted(""), dir.display());
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
    println!("  {} Config and state reset.", style::success(""));
}
