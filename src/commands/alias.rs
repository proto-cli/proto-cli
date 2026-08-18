use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum AliasAction {
    #[command(about = "Create a new shell alias interactively")]
    Create,
    #[command(about = "List all registered Proto aliases")]
    List,
    #[command(about = "Remove a Proto alias")]
    Remove {
        #[arg(required = true, value_name = "NAME")]
        name: String,
    },
}

pub fn run(action: &AliasAction) {
    match action {
        AliasAction::Create => create(),
        AliasAction::List => list(),
        AliasAction::Remove { name } => remove(name),
    }
}

fn create() {
    use dialoguer::{Confirm, Input, MultiSelect};

    println!("{}", style::proto_banner());
    println!("{}\n", "Alias Creator".style(style::Theme::HEADER));

    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Alias name")
        .interact_text()
        .unwrap();

    if name.contains(' ') || name.is_empty() {
        eprintln!("{} Alias name must be a single word.", style::error(""));
        return;
    }

    let command: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Command")
        .interact_text()
        .unwrap();

    let desc: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()
        .unwrap();

    println!();
    let shells = &["bash", "zsh", "fish"];
    let defaults = &[true, true, true];
    let selection = MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Target shells")
        .items(shells)
        .defaults(defaults)
        .interact()
        .unwrap_or_default();

    if selection.is_empty() {
        println!("{} No shells selected.", style::warn(""));
        return;
    }

    let selected_shells: Vec<&str> = selection.iter().map(|&i| shells[i]).collect();

    println!(
        "\n{} {}",
        "◆".style(style::Theme::ACCENT),
        name.style(style::Theme::ACCENT).bold()
    );
    println!("  {} {}", "→".dimmed(), command.dimmed());
    if !desc.is_empty() {
        println!("  {} {}", "  ".dimmed(), desc.style(style::Theme::MUTED));
    }
    println!();

    let permanent = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Save permanently to shell config?")
        .default(true)
        .interact()
        .unwrap_or(true);

    let shell_name = std::env::var("SHELL").unwrap_or_default();
    let current_shell = std::path::Path::new(&shell_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    for sh in &selected_shells {
        let (alias_line, config_path) = match *sh {
            "bash" => {
                let line = if desc.is_empty() {
                    format!("alias {}='{}'", name, command)
                } else {
                    format!("alias {}='{}' # {}", name, command, desc)
                };
                (line, dirs::home_dir().unwrap_or_default().join(".bashrc"))
            }
            "zsh" => {
                let line = if desc.is_empty() {
                    format!("alias {}='{}'", name, command)
                } else {
                    format!("alias {}='{}' # {}", name, command, desc)
                };
                (line, dirs::home_dir().unwrap_or_default().join(".zshrc"))
            }
            "fish" => {
                let line = if desc.is_empty() {
                    format!("alias {}='{}'", name, command)
                } else {
                    format!("alias {} '{}' --description '{}'", name, command, desc)
                };
                (
                    line,
                    dirs::home_dir()
                        .unwrap_or_default()
                        .join(".config/fish/config.fish"),
                )
            }
            _ => continue,
        };

        if permanent {
            append_to_file(&config_path, &format!("\n# proto alias\n{}\n", alias_line));
            println!(
                "{} {} → {} {}",
                "✔".green(),
                sh.style(style::Theme::ACCENT),
                "saved".dimmed(),
                config_path.to_string_lossy().dimmed()
            );
        } else {
            println!(
                "{} {} → {} {}",
                "✦".cyan(),
                sh.style(style::Theme::ACCENT),
                "session only".dimmed(),
                format!("source <(echo '{}')", alias_line).dimmed()
            );
            if *sh == current_shell {
                let source_cmd = if current_shell == "fish" {
                    format!("alias {} '{}'", name, command)
                } else {
                    format!("alias {}='{}'", name, command)
                };
                println!(
                    "  {}  Run: {}",
                    "  ".dimmed(),
                    source_cmd.style(style::Theme::ACCENT)
                );
            }
        }
    }

    println!(
        "\n{} Alias '{}' configured for: {}",
        style::success(""),
        name.style(style::Theme::ACCENT),
        selected_shells.join(", ").style(style::Theme::MUTED)
    );
    if permanent {
        println!(
            "  Restart your shell or run: {}",
            "source <config>".style(style::Theme::MUTED)
        );
    }
}

fn list() {
    let home = dirs::home_dir().unwrap_or_default();
    let configs = [
        ("bash", home.join(".bashrc")),
        ("zsh", home.join(".zshrc")),
        ("fish", home.join(".config/fish/config.fish")),
    ];

    let mut found = false;
    for (shell, path) in &configs {
        if !path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut in_proto = false;
        for line in content.lines() {
            if line.contains("# proto alias") {
                in_proto = true;
                if !found {
                    println!("{}", "Proto Aliases".style(style::Theme::HEADER));
                    println!("{}", style::divider());
                }
                found = true;
                continue;
            }
            if in_proto && (line.starts_with("alias ") || line.starts_with("abbr ")) {
                println!(
                    "  {} {} {}",
                    "▸".style(style::Theme::ACCENT),
                    shell.style(style::Theme::MUTED),
                    line.trim().style(style::Theme::MUTED)
                );
                in_proto = false;
            }
        }
    }

    if !found {
        println!("{} No proto aliases found.", "  ".dimmed());
        println!("  {}", "proto alias create".style(style::Theme::MUTED));
    } else {
        println!("{}", style::divider());
    }
}

fn remove(name: &str) {
    use dialoguer::Confirm;
    let home = dirs::home_dir().unwrap_or_default();
    let configs = [
        ("bash", home.join(".bashrc")),
        ("zsh", home.join(".zshrc")),
        ("fish", home.join(".config/fish/config.fish")),
    ];

    let mut removed = 0;
    for (_shell, path) in &configs {
        if !path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut new_lines = Vec::new();
        let mut skip = false;

        for line in content.lines() {
            if line.contains("# proto alias")
                && new_lines
                    .last()
                    .map(|l: &String| l.contains(name))
                    .unwrap_or(false)
            {
                new_lines.pop();
                skip = true;
                removed += 1;
                continue;
            }
            if skip {
                skip = false;
                continue;
            }
            new_lines.push(line.to_string());
        }

        let new_content = new_lines.join("\n") + "\n";
        if new_content != content {
            let confirm = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt(format!(
                    "Remove '{}' from {}?",
                    name,
                    path.to_string_lossy()
                ))
                .default(true)
                .interact()
                .unwrap_or(false);
            if confirm {
                std::fs::write(path, &new_content).unwrap();
            }
        }
    }

    if removed > 0 {
        println!(
            "{} Removed '{}' from {} file(s).",
            style::success(""),
            name.style(style::Theme::ACCENT),
            removed
        );
    } else {
        println!("{} Alias '{}' not found.", style::warn(""), name);
    }
}

fn append_to_file(path: &std::path::Path, content: &str) {
    let existing = if path.exists() {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };
    if !existing.contains(content.trim()) {
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::write(path, format!("{}{}", existing, content)).unwrap();
    }
}
