use crate::commands;
use crate::plugins;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "proto",
    version,
    about = "Your friendly protogen CLI companion",
    long_about = None,
    disable_help_subcommand = true,
    disable_help_flag = true,
    disable_version_flag = true,
    arg_required_else_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue, global = false)]
    pub help_flag: bool,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue, global = false)]
    pub version_flag: bool,

    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        help = "Suppress non-essential output"
    )]
    pub quiet: bool,

    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(about = "Show help for all commands or a specific command")]
    Help {
        #[arg(value_name = "COMMAND")]
        command: Option<String>,
    },
    #[command(about = "Display beautiful system information")]
    System,
    #[command(about = "Cross-distro package manager wrapper")]
    Pkg {
        #[command(subcommand)]
        action: commands::pkg::PkgAction,
    },
    #[command(about = "Git workflow enhancements")]
    Git {
        #[command(subcommand)]
        action: commands::git::GitAction,
    },
    #[command(about = "Interactive first-time configuration wizard")]
    Setup,
    #[command(about = "Interactive shell alias builder (bash/zsh/fish)")]
    Alias {
        #[command(subcommand)]
        action: commands::alias::AliasAction,
    },
    #[command(about = "Manage the Proto CLI itself (update, uninstall, reset)")]
    Manage {
        #[command(subcommand)]
        action: commands::manage::ManageAction,
    },
    #[command(about = "Manage proto plugins")]
    Plugins {
        #[command(subcommand)]
        action: commands::plugins::PluginsAction,
    },
    #[command(about = "Generate shell completion scripts")]
    Completions {
        #[arg(value_name = "SHELL", help = "Shell to generate for (bash, zsh, fish)")]
        shell: String,

        #[arg(long, help = "Install completions to proto config dir")]
        install: bool,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

pub fn run(cli: Cli) {
    if cli.version_flag {
        print_version();
        return;
    }

    if cli.help_flag {
        print_short_help();
        return;
    }

    match cli.command {
        Some(Commands::Help { command }) => match command {
            Some(cmd) => commands::help::run(&commands::help::HelpAction::For { command: cmd }),
            None => commands::help::run(&commands::help::HelpAction::All),
        },
        Some(Commands::System) => commands::system::run(),
        Some(Commands::Pkg { action }) => commands::pkg::run(&action),
        Some(Commands::Git { action }) => commands::git::run(&action),
        Some(Commands::Setup) => commands::setup::run(),
        Some(Commands::Alias { action }) => commands::alias::run(&action),
        Some(Commands::Manage { action }) => commands::manage::run(&action),
        Some(Commands::Plugins { action }) => commands::plugins::run(&action),
        Some(Commands::Completions { shell, install }) => {
            if install {
                crate::completions::install_completions();
            } else {
                crate::completions::generate(&shell);
            }
        }
        Some(Commands::External(args)) => {
            if let Some(command) = args.first() {
                let (scope, name) = plugins::parse_plugin_ref(command);
                if let Some(binary) = plugins::discovery::find_plugin_binary(&scope, &name) {
                    let plugin_args: Vec<String> = args[1..].to_vec();
                    if let Err(e) = plugins::execute::execute_plugin(&binary, &plugin_args) {
                        eprintln!("Plugin error: {}", e);
                        std::process::exit(1);
                    }
                } else if let Some((_info, binary)) =
                    plugins::discovery::find_plugin_for_command(command)
                {
                    let plugin_args: Vec<String> = args[1..].to_vec();
                    if let Err(e) = plugins::execute::execute_plugin(&binary, &plugin_args) {
                        eprintln!("Plugin error: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Unknown command: {}", command);
                    eprintln!("Run 'proto help' to see available commands.");
                    eprintln!("Run 'proto plugins list' to see installed plugins.");
                    std::process::exit(1);
                }
            }
        }
        None => {
            commands::help::run(&commands::help::HelpAction::All);
        }
    }
}

fn print_version() {
    use crate::style;
    use owo_colors::OwoColorize;

    println!("{}", style::proto_banner());
    println!(
        "{} {}",
        "proto".style(style::Theme::HEADER).bold(),
        env!("CARGO_PKG_VERSION").style(style::Theme::MUTED)
    );
    println!(
        "{}",
        "Your friendly protogen CLI companion".style(style::Theme::MUTED)
    );
}

fn print_short_help() {
    commands::help::run(&commands::help::HelpAction::All);
}
