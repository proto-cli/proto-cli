mod cli;
mod commands;
mod completions;
mod globals;
mod plugins;
mod style;
mod update;
mod utils;

use clap::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--quiet" || a == "-q") {
        globals::set_quiet(true);
    }
    if args.iter().any(|a| a == "--json") {
        globals::set_json(true);
    }
    if args.iter().any(|a| a == "--no-color") {
        std::env::set_var("NO_COLOR", "1");
    }

    if !globals::is_quiet() {
        std::thread::spawn(update::check_for_update);
    }

    let cli = cli::Cli::parse();
    cli::run(cli);
}
