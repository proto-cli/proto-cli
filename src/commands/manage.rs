use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;
use std::process::Command;

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

fn update() {
    println!("{}", style::header("Proto Update"));
    println!("{}", style::divider());

    let repo = repo_dir();
    println!("  {}\n", style::muted(&format!("Repo: {}", repo)));

    let spin = style::Spinner::new("git pull origin master...");
    let pull = Command::new("git")
        .args(["-C", repo, "pull", "origin", "master"])
        .output();
    match pull {
        Ok(o) if o.status.success() => {
            spin.done("Git pull complete");
            let stdout = String::from_utf8_lossy(&o.stdout);
            if !stdout.contains("Already up to date") {
                println!("  {} New commits pulled.", style::muted(""));
            }
        }
        Ok(o) => {
            spin.fail("Git pull failed");
            eprintln!("  {}", String::from_utf8_lossy(&o.stderr));
            return;
        }
        Err(e) => {
            spin.fail(&format!("Git pull error: {}", e));
            return;
        }
    }

    let spin = style::Spinner::new("cargo build --release...");
    let build = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(repo)
        .output();
    match build {
        Ok(o) if o.status.success() => {
            spin.done("Build complete");
        }
        Ok(o) => {
            spin.fail("Build failed");
            eprintln!("  {}", String::from_utf8_lossy(&o.stderr));
            return;
        }
        Err(e) => {
            spin.fail(&format!("Build error: {}", e));
            return;
        }
    }

    let current = std::env::current_exe().unwrap_or_default();
    let new_binary = format!("{}/target/release/proto", repo);
    println!(
        "  {} Installing {} -> {}",
        style::muted(""),
        new_binary,
        current.display()
    );

    // Writing to a running executable fails with ETXTBSY ("Text file busy"),
    // so stage the new binary next to the target and atomically rename it
    // into place instead.
    let mut tmp_name = current
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp = current.with_file_name(tmp_name);

    if std::fs::copy(&new_binary, &tmp)
        .and_then(|_| std::fs::rename(&tmp, &current))
        .is_err()
    {
        let _ = std::fs::remove_file(&tmp);
        // Fallback: unlink the busy binary first, then copy fresh.
        match std::fs::remove_file(&current).and_then(|_| std::fs::copy(&new_binary, &current)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("  {} Failed to install: {}", style::error(""), e);
                return;
            }
        }
    }

    println!("  {} Update complete.", style::success(""));
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
