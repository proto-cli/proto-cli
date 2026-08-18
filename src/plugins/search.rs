use crate::plugins;
use crate::style;
use owo_colors::OwoColorize;

const GITHUB_API_TREE: &str = "https://api.github.com/repos/proto-cli/plugins/contents/";

pub fn search_and_install() {
    println!(
        "{}",
        "Fetching available plugins...".style(style::Theme::MUTED)
    );

    let plugin_dirs = match fetch_plugin_dirs() {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!(
                "{}",
                format!("Failed to fetch plugin list: {}", e).style(style::Theme::ERROR)
            );
            return;
        }
    };

    if plugin_dirs.is_empty() {
        println!(
            "{}",
            "No plugins available in the registry.".style(style::Theme::MUTED)
        );
        return;
    }

    let installed = plugins::discovery::scan_installed_plugins();
    let installed_names: Vec<String> = installed
        .iter()
        .map(|(info, _)| info.name.clone())
        .collect();

    let mut items: Vec<String> = plugin_dirs
        .iter()
        .map(|name| {
            let status = if installed_names.contains(name) {
                " (installed)"
            } else {
                ""
            };
            format!("{}{}", name, status)
        })
        .collect();

    items.insert(0, "← Back".to_string());

    let selection = dialoguer::FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Search plugins")
        .items(&items)
        .default(0)
        .interact_opt();

    match selection {
        Ok(Some(idx)) => {
            if idx == 0 {
                return;
            }
            let selected = &plugin_dirs[idx - 1];
            if let Err(e) = plugins::install::install_plugin(selected) {
                eprintln!("{}", format!("Error: {}", e).style(style::Theme::ERROR));
            }
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!(
                "{}",
                format!("Selection error: {}", e).style(style::Theme::ERROR)
            );
        }
    }
}

pub fn list_available() {
    println!(
        "{}",
        "Fetching available plugins...".style(style::Theme::MUTED)
    );

    let plugin_dirs = match fetch_plugin_dirs() {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!(
                "{}",
                format!("Failed to fetch plugin list: {}", e).style(style::Theme::ERROR)
            );
            return;
        }
    };

    if plugin_dirs.is_empty() {
        println!(
            "{}",
            "No plugins available in the registry.".style(style::Theme::MUTED)
        );
        return;
    }

    let installed = plugins::discovery::scan_installed_plugins();
    let installed_names: Vec<String> = installed
        .iter()
        .map(|(info, _)| info.name.clone())
        .collect();

    println!(
        "\n{}",
        "Available Plugins:".style(style::Theme::HEADER).bold()
    );
    println!();

    for name in &plugin_dirs {
        let is_installed = installed_names.contains(name);
        let marker = if is_installed {
            " ✔".style(style::Theme::SUCCESS).to_string()
        } else {
            String::new()
        };

        println!(
            "  {} {}{}",
            "@proto/".to_string().style(style::Theme::ACCENT),
            name.style(style::Theme::HEADER).bold(),
            marker
        );
    }

    println!(
        "\n{}",
        "Run 'proto plugins search' to install interactively.".style(style::Theme::MUTED)
    );
}

fn fetch_plugin_dirs() -> Result<Vec<String>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("proto-cli")
        .build();

    let resp = agent
        .get(GITHUB_API_TREE)
        .call()
        .map_err(|e| format!("GitHub API error: {}", e))?;

    let body: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let dirs: Vec<String> = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let name = item["name"].as_str()?;
                    let item_type = item["type"].as_str()?;
                    if item_type == "dir" && !name.starts_with('.') && name != "target" {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(dirs)
}
