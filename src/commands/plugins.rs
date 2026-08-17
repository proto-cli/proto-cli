use crate::plugins;
use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum PluginsAction {
    #[command(about = "List all available and installed plugins")]
    List,
    #[command(about = "Install a plugin")]
    Add {
        #[arg(value_name = "PLUGIN", help = "Plugin name (e.g. mc-server, @proto/mc-server)")]
        plugin: String,
    },
    #[command(about = "Remove an installed plugin")]
    Remove {
        #[arg(value_name = "PLUGIN", help = "Plugin name to remove")]
        plugin: String,
    },
    #[command(about = "Update plugins")]
    Update {
        #[arg(value_name = "PLUGIN", help = "Plugin to update (omit to update all)")]
        plugin: Option<String>,
    },
    #[command(about = "Add a custom plugin repository")]
    AddRepo {
        #[arg(value_name = "URL", help = "Git repository URL")]
        url: String,
    },
}

pub fn run(action: &PluginsAction) {
    match action {
        PluginsAction::List => list_plugins(),
        PluginsAction::Add { plugin } => {
            if let Err(e) = plugins::install::install_plugin(plugin) {
                println!("{}", format!("Error: {}", e).style(style::Theme::ERROR));
            }
        }
        PluginsAction::Remove { plugin } => {
            if let Err(e) = plugins::install::remove_plugin(plugin) {
                println!("{}", format!("Error: {}", e).style(style::Theme::ERROR));
            }
        }
        PluginsAction::Update { plugin } => {
            if let Err(e) = plugins::install::update_plugin(plugin.as_deref()) {
                println!("{}", format!("Error: {}", e).style(style::Theme::ERROR));
            }
        }
        PluginsAction::AddRepo { url } => add_repo(url),
    }
}

fn list_plugins() {
    let discovered = plugins::discovery::scan_installed_plugins();

    println!(
        "{}",
        "Installed Plugins:".style(style::Theme::HEADER).bold()
    );
    println!();

    if discovered.is_empty() {
        println!(
            "  {}",
            "No plugins installed.".style(style::Theme::MUTED)
        );
        println!(
            "  {}",
            "Run 'proto plugins add <name>' to install a plugin.".style(style::Theme::MUTED)
        );
    } else {
        for (info, _binary) in &discovered {
            let commands_str: Vec<String> = info.commands.keys().cloned().collect();
            println!(
                "  {} {} {}",
                format!("@{}/", info.scope).style(style::Theme::ACCENT),
                info.name.style(style::Theme::HEADER).bold(),
                format!("v{}", info.version).style(style::Theme::MUTED)
            );
            println!(
                "    {} {}",
                "Commands:".style(style::Theme::MUTED),
                commands_str.join(", ").style(style::Theme::SUCCESS)
            );
            println!(
                "    {}",
                info.description.style(style::Theme::MUTED)
            );
        }
    }

    println!();
    println!(
        "{}",
        "Get more plugins:".style(style::Theme::HEADER).bold()
    );
    println!(
        "  {}",
        "https://github.com/proto-cli/plugins".style(style::Theme::MUTED)
    );
    println!(
        "  {}",
        "proto plugins add <name>  - Install a plugin".style(style::Theme::MUTED)
    );
    println!(
        "  {}",
        "Community plugins: @username/plugin-name".style(style::Theme::MUTED)
    );
}

fn add_repo(url: &str) {
    let mut config = crate::utils::load_config();
    let mut repos = config.custom_repos.unwrap_or_default();
    if repos.contains(&url.to_string()) {
        println!(
            "{}",
            format!("Repository already added: {}", url).style(style::Theme::WARN)
        );
        return;
    }
    repos.push(url.to_string());
    config.custom_repos = Some(repos);
    if let Err(e) = crate::utils::save_config(&config) {
        println!("{}", format!("Error: {}", e).style(style::Theme::ERROR));
    } else {
        println!(
            "{}",
            format!("Added repository: {}", url).style(style::Theme::SUCCESS)
        );
    }
}
