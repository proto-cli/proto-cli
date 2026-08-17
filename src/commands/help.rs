use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum HelpAction {
    #[command(about = "Show help for all commands")]
    All,
    #[command(about = "Show help for a specific command")]
    For {
        #[arg(value_name = "COMMAND")]
        command: String,
    },
}

pub fn run(action: &HelpAction) {
    match action {
        HelpAction::All => print_general_help(),
        HelpAction::For { command } => print_command_help(command),
    }
}

fn print_general_help() {
    println!("{}", style::proto_banner());
    println!("{}\n", "Proto CLI".style(style::Theme::HEADER).bold());
    println!(
        "{}  {}",
        "◆".style(style::Theme::ACCENT),
        "Your friendly protogen CLI companion\n".style(style::Theme::MUTED)
    );

    println!("{}", "USAGE:".style(style::Theme::HEADER));
    println!(
        "  {} <command> [options]\n",
        "proto".style(style::Theme::ACCENT)
    );

    println!("{}", "COMMANDS:".style(style::Theme::HEADER));
    print_cmd(
        "help",
        "[command]",
        "Show this help or help for a specific command",
    );
    print_cmd("system", "", "Display beautiful system information");
    print_cmd("pkg", "<action>", "Cross-distro package manager wrapper");
    print_cmd("git", "<action>", "Git workflow enhancements");
    print_cmd("setup", "", "Interactive configuration wizard");
    print_cmd(
        "alias",
        "create|list|remove",
        "Interactive shell alias builder",
    );
    print_cmd(
        "manage",
        "update|uninstall|reset",
        "Manage the Proto CLI itself",
    );
    print_cmd("plugins", "list|add|remove|update", "Manage proto plugins");

    let installed = crate::plugins::registry::list_plugins();
    let plugin_cmds: Vec<_> = installed.iter().filter(|p| p.installed).collect();
    if !plugin_cmds.is_empty() {
        println!("\n{}", "PLUGINS:".style(style::Theme::HEADER));
        for plugin in &plugin_cmds {
            for cmd in &plugin.commands {
                print_cmd(cmd, "", &format!("@{}/{}", plugin.scope, plugin.name));
            }
        }
    }

    println!(
        "\n{}",
        "Run 'proto plugins list' to see available plugins.".style(style::Theme::MUTED)
    );

    println!("\n{}", "FLAGS:".style(style::Theme::HEADER));
    print_cmd("--version", "", "Print version and exit");
    print_cmd("--help", "", "Print help information");

    println!(
        "\n{} {}\n",
        "Run".style(style::Theme::MUTED),
        "'proto help <command>' for more info.".style(style::Theme::MUTED)
    );
}

fn print_command_help(command: &str) {
    match command.to_lowercase().as_str() {
        "help" => {
            println!("{}", "proto help".style(style::Theme::HEADER));
            println!("  Display help for all commands or a specific command.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto help              Show general help");
            println!("  proto help <command>    Show help for a command\n");
            println!("{}", "EXAMPLES:".style(style::Theme::HEADER));
            println!("  proto help system       Show system command help");
            println!("  proto help pkg          Show package manager help");
        }
        "system" => {
            println!("{}", "proto system".style(style::Theme::HEADER));
            println!("  Display a beautiful overview of your system.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto system\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  OS, kernel, architecture, CPU, RAM, disk usage,");
            println!("  uptime, DE/WM, shell, terminal, and package count.");
        }
        "alias" => {
            println!("{}", "proto alias".style(style::Theme::HEADER));
            println!("  Interactive shell alias builder for bash, zsh, and fish.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto alias create    Build a new alias interactively");
            println!("  proto alias list      Show all Proto-managed aliases");
            println!("  proto alias remove <NAME>  Remove an alias\n");
            println!("{}", "CREATE FLOW:".style(style::Theme::HEADER));
            println!("  1. Enter alias name + command + description");
            println!("  2. Choose target shells (multi-select)");
            println!("  3. Choose permanent (writes to .bashrc/.zshrc/config.fish)");
            println!("     or session-only");
        }
        "pkg" => {
            println!("{}", "proto pkg".style(style::Theme::HEADER));
            println!("  Unified cross-distro package manager wrapper.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto pkg install <pkg>     Install a package");
            println!("  proto pkg search <query>    Search repositories");
            println!("  proto pkg remove <pkg>      Remove a package");
            println!("  proto pkg update [pkg]      Update all or specific packages");
            println!("  proto pkg list              List installed packages\n");
            println!("{}", "SUPPORTED:".style(style::Theme::HEADER));
            println!("  pacman, yay, paru, apt, dnf, zypper, apk");
        }
        "git" => {
            println!("{}", "proto git".style(style::Theme::HEADER));
            println!("  Git workflow enhancements.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto git log               Pretty git log with graph");
            println!("  proto git stats             Repository statistics");
            println!("  proto git save <msg>        Quick WIP commit (add all + commit)");
            println!("  proto git undo              Undo last commit (soft reset)");
            println!("  proto git branch            Show branches with info");
        }
        "setup" => {
            println!("{}", "proto setup".style(style::Theme::HEADER));
            println!("  Interactive first-time configuration wizard.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto setup\n");
            println!("{}", "CONFIGURES:".style(style::Theme::HEADER));
            println!("  Default package manager, color preferences,");
            println!("  shell completions, and more.");
        }
        "manage" => {
            println!("{}", "proto manage".style(style::Theme::HEADER));
            println!("  Manage the Proto CLI itself.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto manage update                 Git pull + cargo build --release");
            println!("  proto manage uninstall              Remove the proto binary");
            println!("  proto manage uninstall --purge      Also delete the cloned repo");
            println!("  proto manage reset                  Remove proto config and state\n");
            println!("{}", "DETAILS:".style(style::Theme::HEADER));
            println!("  Uses the compile-time repo path (env! CARGO_MANIFEST_DIR)");
            println!("  and updates the binary at this process's current_exe().");
        }
        "plugins" => {
            println!("{}", "proto plugins".style(style::Theme::HEADER));
            println!("  Install, update, and manage proto plugins.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto plugins list                    List all installed plugins");
            println!("  proto plugins add @scope/name         Install a plugin");
            println!("  proto plugins remove @scope/name      Uninstall a plugin");
            println!("  proto plugins update @scope/name      Update a plugin");
            println!("  proto plugins add-repo <url>          Add a custom plugin repository\n");
            println!("{}", "INFO:".style(style::Theme::HEADER));
            println!("  Plugins are pre-compiled binaries installed to ~/.config/proto/plugins/");
            println!("  Registry is synced from GitHub: proto-cli/plugins");
        }
        other => {
            let installed = crate::plugins::registry::list_plugins();
            for plugin in &installed {
                if plugin.installed && plugin.commands.iter().any(|c| c == other) {
                    if let Some(binary) = crate::plugins::discovery::find_plugin_binary(&plugin.scope, &plugin.name) {
                        if let Err(e) = crate::plugins::execute::execute_plugin(&binary, &["--help".to_string()]) {
                            eprintln!("Plugin error: {}", e);
                        }
                        return;
                    }
                }
            }
            println!(
                "{} Unknown command: '{}'",
                style::error(""),
                other.style(style::Theme::ACCENT)
            );
            println!(
                "Run {} to see all available commands.",
                "proto help".style(style::Theme::ACCENT)
            );
        }
    }
}

fn print_cmd(name: &str, args: &str, desc: &str) {
    let name_part = if args.is_empty() {
        format!("  {}          ", name.style(style::Theme::ACCENT))
    } else {
        format!(
            "  {} {}",
            name.style(style::Theme::ACCENT),
            args.style(style::Theme::BOLD)
        )
    };
    println!(
        "{}{}",
        format!("{:40}", name_part),
        desc.style(style::Theme::MUTED)
    );
}
