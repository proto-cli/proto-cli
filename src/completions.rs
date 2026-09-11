use crate::cli::Cli;
use crate::style;
use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::generate as gen_completion;
use owo_colors::OwoColorize;
use std::fs;

fn parse_shell(shell: &str) -> Option<Shell> {
    match shell {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        _ => None,
    }
}

fn script(shell: Shell) -> String {
    let mut command = Cli::command();
    let mut buf = Vec::new();
    gen_completion(shell, &mut command, "proto", &mut buf);
    String::from_utf8_lossy(&buf).to_string()
}

pub fn generate(shell: &str) {
    match parse_shell(shell) {
        Some(sh) => print!("{}", script(sh)),
        None => {
            eprintln!(
                "{}",
                format!("Unsupported shell: {}", shell).style(style::Theme::ERROR)
            );
            eprintln!(
                "{}",
                "Supported shells: bash, zsh, fish".style(style::Theme::MUTED)
            );
        }
    }
}

pub fn install_completions() {
    let config_dir = crate::utils::config_dir();
    let comp_dir = config_dir.join("completions");
    let _ = fs::create_dir_all(&comp_dir);

    let bash_path = comp_dir.join("proto.bash");
    let zsh_path = comp_dir.join("proto.zsh");
    let fish_dir = comp_dir.join("fish");
    let fish_path = fish_dir.join("proto.fish");

    let _ = fs::write(&bash_path, script(Shell::Bash));
    let _ = fs::write(&zsh_path, script(Shell::Zsh));
    let _ = fs::create_dir_all(&fish_dir);
    let _ = fs::write(&fish_path, script(Shell::Fish));

    println!(
        "{}",
        "Shell completions generated:"
            .style(style::Theme::HEADER)
            .bold()
    );
    println!();
    println!(
        "  {} {}",
        "Bash:".style(style::Theme::MUTED),
        bash_path.display()
    );
    println!(
        "    {}",
        format!("source {}", bash_path.display()).style(style::Theme::ACCENT)
    );
    println!();
    println!(
        "  {} {}",
        "Zsh:".style(style::Theme::MUTED),
        zsh_path.display()
    );
    println!(
        "    {}",
        format!("fpath=({} $fpath)", comp_dir.display()).style(style::Theme::ACCENT)
    );
    println!(
        "    {}",
        "autoload -Uz compinit && compinit".style(style::Theme::ACCENT)
    );
    println!();
    println!(
        "  {} {}",
        "Fish:".style(style::Theme::MUTED),
        fish_path.display()
    );
    println!(
        "    {}",
        format!("fish_add_path {}", comp_dir.display()).style(style::Theme::ACCENT)
    );
    println!(
        "    {}",
        format!(
            "cp {} ~/.config/fish/completions/proto.fish",
            fish_path.display()
        )
        .style(style::Theme::ACCENT)
    );

    let mut config = crate::utils::load_config();
    config.completions_installed = Some(true);
    let _ = crate::utils::save_config(&config);

    println!(
        "\n{}",
        "Add the source commands to your shell rc file for persistent completions."
            .style(style::Theme::MUTED)
    );
}