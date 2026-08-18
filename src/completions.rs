use crate::style;
use owo_colors::OwoColorize;
use std::fs;

const BASH_COMPLETION: &str = r#"#!/usr/bin/env bash
# Proto CLI shell completions

_proto_completions() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="help system pkg git setup alias manage plugins completions"
    local plugins=""

    if [[ -d "$HOME/.config/proto/plugins" ]]; then
        for dir in "$HOME/.config/proto/plugins"/*/; do
            [[ -d "$dir" ]] || continue
            local name=$(basename "$dir")
            name="${name#proto-}"
            plugins="$plugins $name"
        done
    fi

    local all="$commands $plugins"

    if [[ ${cur} == -* ]]; then
        COMPREPLY=($(compgen -W "--help --version -h -V" -- "${cur}"))
        return 0
    fi

    COMPREPLY=($(compgen -W "${all}" -- "${cur}"))
    return 0
}

complete -F _proto_completions proto
"#;

const ZSH_COMPLETION: &str = r#"#compdef proto

# Proto CLI shell completions

_proto() {
    local -a commands plugins

    commands=(
        'help:Show help for all commands or a specific command'
        'system:Display beautiful system information'
        'pkg:Cross-distro package manager wrapper'
        'git:Git workflow enhancements'
        'setup:Interactive first-time configuration wizard'
        'alias:Interactive shell alias builder'
        'manage:Manage the Proto CLI itself'
        'plugins:Manage proto plugins'
        'completions:Generate shell completion scripts'
    )

    plugins=()
    if [[ -d "$HOME/.config/proto/plugins" ]]; then
        for dir in "$HOME/.config/proto/plugins"/*/; do
            [[ -d "$dir" ]] || continue
            local name=$(basename "$dir")
            name="${name#proto-}"
            plugins+=("$name")
        done
    fi

    _arguments -C \
        '1:command:->cmd' \
        '*::arg:->args'

    case $state in
        cmd)
            _describe -t commands 'proto command' commands
            if (( ${#plugins} )); then
                _describe -t plugins 'proto plugin' plugins
            fi
            ;;
        args)
            case ${words[1]} in
                help)
                    _arguments '1:command:_proto_commands' ;;
                pkg)
                    _arguments '1:action:(install search remove update list)' '2:package:_packages' ;;
                git)
                    _arguments '1:action:(log stats save undo branch)' ;;
                alias)
                    _arguments '1:action:(create list remove)' ;;
                manage)
                    _arguments '1:action:(update uninstall reset)' ;;
                plugins)
                    _arguments '1:action:(list add remove update search add-repo)' ;;
                completions)
                    _arguments '1:shell:(bash zsh fish)' '--install:Install completions' ;;
            esac
            ;;
    esac
}

_proto_commands() {
    local -a cmds
    cmds=(
        'help:Show help for all commands or a specific command'
        'system:Display beautiful system information'
        'pkg:Cross-distro package manager wrapper'
        'git:Git workflow enhancements'
        'setup:Interactive first-time configuration wizard'
        'alias:Interactive shell alias builder'
        'manage:Manage the Proto CLI itself'
        'plugins:Manage proto plugins'
        'completions:Generate shell completion scripts'
    )
    _describe -t commands 'command' cmds
}

_proto "$@"
"#;

const FISH_COMPLETION: &str = r#"# Proto CLI shell completions

function __proto_get_commands
    echo help
    echo system
    echo pkg
    echo git
    echo setup
    echo alias
    echo manage
    echo plugins
    echo completions

    if test -d "$HOME/.config/proto/plugins"
        for dir in "$HOME/.config/proto/plugins"/*/
            set -l name (basename "$dir")
            string replace -r '^proto-' '' "$name"
        end
    end
end

function __proto_pkg_actions
    echo install
    echo search
    echo remove
    echo update
    echo list
end

function __proto_git_actions
    echo log
    echo stats
    echo save
    echo undo
    echo branch
end

function __proto_alias_actions
    echo create
    echo list
    echo remove
end

function __proto_manage_actions
    echo update
    echo uninstall
    echo reset
end

function __proto_plugins_actions
    echo list
    echo add
    echo remove
    echo update
    echo search
    echo add-repo
end

function __proto_completions_shells
    echo bash
    echo zsh
    echo fish
end

complete -c proto -f
complete -c proto -n '__fish_use_subcommand' -a '(__proto_get_commands)' -d 'Command'

complete -c proto -n '__fish_seen_subcommand_from help' -a '(__proto_get_commands)' -d 'Command'
complete -c proto -n '__fish_seen_subcommand_from pkg' -a '(__proto_pkg_actions)' -d 'Action'
complete -c proto -n '__fish_seen_subcommand_from git' -a '(__proto_git_actions)' -d 'Action'
complete -c proto -n '__fish_seen_subcommand_from alias' -a '(__proto_alias_actions)' -d 'Action'
complete -c proto -n '__fish_seen_subcommand_from manage' -a '(__proto_manage_actions)' -d 'Action'
complete -c proto -n '__fish_seen_subcommand_from plugins' -a '(__proto_plugins_actions)' -d 'Action'
complete -c proto -n '__fish_seen_subcommand_from completions' -a '(__proto_completions_shells)' -d 'Shell'
"#;

pub fn generate(shell: &str) {
    match shell {
        "bash" => print!("{}", BASH_COMPLETION),
        "zsh" => print!("{}", ZSH_COMPLETION),
        "fish" => print!("{}", FISH_COMPLETION),
        _ => {
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

    let _ = fs::write(&bash_path, BASH_COMPLETION);
    let _ = fs::write(&zsh_path, ZSH_COMPLETION);
    let _ = fs::create_dir_all(&fish_dir);
    let _ = fs::write(&fish_path, FISH_COMPLETION);

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
