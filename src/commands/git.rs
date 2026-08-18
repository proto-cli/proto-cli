use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum GitAction {
    #[command(about = "Show a pretty git log with graph")]
    Log {
        #[arg(short = 'n', long, default_value = "10", value_name = "N")]
        count: usize,
    },
    #[command(about = "Show repository statistics")]
    Stats,
    #[command(about = "Quick WIP commit (stage all + commit)")]
    Save {
        #[arg(required = true, value_name = "MESSAGE")]
        message: String,
    },
    #[command(about = "Undo last commit keeping changes staged")]
    Undo,
    #[command(about = "Show branches with last commit info")]
    Branch,
}

pub fn run(action: &GitAction) {
    use crate::style;
    use crate::utils;

    if !utils::which("git") {
        eprintln!("{}", style::error("Git is not installed."));
        std::process::exit(1);
    }

    if !is_git_repo() {
        eprintln!("{}", style::error("Not inside a git repository."));
        std::process::exit(1);
    }

    let result = match action {
        GitAction::Log { count } => git_log(*count),
        GitAction::Stats => git_stats(),
        GitAction::Save { message } => git_save(message),
        GitAction::Undo => git_undo(),
        GitAction::Branch => git_branch(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style::error(""), e);
        std::process::exit(1);
    }
}

fn is_git_repo() -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn git_log(count: usize) -> Result<(), String> {
    use crate::style;

    let format_str = "--format=format:%h %<(20,trunc)%an %s %C(auto)%d";
    let n_str = format!("-n{}", count);
    let args: Vec<&str> = vec![
        "log",
        "--graph",
        "--color=always",
        n_str.as_str(),
        format_str,
    ];

    let output = crate::utils::run_command_output("git", &args)
        .map_err(|e| format!("Failed to show git log: {}", e))?;

    if output.is_empty() {
        println!("{}", "No commits yet.".style(style::Theme::MUTED));
        return Ok(());
    }

    println!("{}", "Commits".style(style::Theme::HEADER));
    println!("{}", style::divider());

    for line in output.lines() {
        println!("{}", line);
    }

    println!("{}", style::divider());
    Ok(())
}

fn git_stats() -> Result<(), String> {
    use crate::style;
    use owo_colors::OwoColorize;

    println!("{}", "Repository Stats".style(style::Theme::HEADER));
    println!("{}", style::divider());

    if let Ok(branch) = crate::utils::run_command_output("git", &["branch", "--show-current"]) {
        println!("{}", style::label_value("Branch", &branch));
    }

    if let Ok(remote) = crate::utils::run_command_output("git", &["remote", "get-url", "origin"]) {
        println!("{}", style::label_value("Remote", &remote));
    }

    if let Ok(commits) = crate::utils::run_command_output("git", &["rev-list", "--count", "HEAD"]) {
        println!("{}", style::label_value("Commits", &commits));
    }

    if let Ok(contributors) = crate::utils::run_command_output("git", &["shortlog", "-sn", "HEAD"])
    {
        let count = contributors.lines().count();
        println!("{}", style::label_value("Contributors", &count.to_string()));
    }

    if let Ok(files) = crate::utils::run_command_output("git", &["ls-files"]) {
        let count = files.lines().count();
        println!(
            "{}",
            style::label_value("Tracked files", &count.to_string())
        );
    }

    if let Ok(modified) = crate::utils::run_command_output("git", &["status", "--porcelain"]) {
        let m = modified.lines().filter(|l| !l.is_empty()).count();
        let status_text = if m == 0 {
            "clean".style(style::Theme::SUCCESS).to_string()
        } else {
            format!("{} files", m).style(style::Theme::WARN).to_string()
        };
        println!("{}", style::label_value("Status", &status_text));
    }

    println!("{}", style::divider());
    Ok(())
}

fn git_save(message: &str) -> Result<(), String> {
    use crate::style;

    let spinner = style::Spinner::new("Staging all changes...");
    crate::utils::run_command("git", &["add", "."])
        .map_err(|e| format!("git add failed: {}", e))?;
    spinner.done("Staged all changes");

    let spinner = style::Spinner::new(&format!("Committing: {}", message));
    crate::utils::run_command("git", &["commit", "-m", message])
        .map_err(|e| format!("git commit failed: {}", e))?;
    spinner.done(&format!("Committed: {}", message));

    println!("{}", style::success("WIP commit saved!"));
    Ok(())
}

fn git_undo() -> Result<(), String> {
    use crate::style;

    let spinner = style::Spinner::new("Undoing last commit...");
    crate::utils::run_command("git", &["reset", "--soft", "HEAD~1"])
        .map_err(|e| format!("git reset failed: {}", e))?;
    spinner.done("Undid last commit");

    println!(
        "{} Changes are still staged. Edit and re-commit.",
        style::success("Last commit undone!")
    );
    Ok(())
}

fn git_branch() -> Result<(), String> {
    use crate::style;
    use owo_colors::OwoColorize;

    println!("{}", "Branches".style(style::Theme::HEADER));
    println!("{}", style::divider());

    let current =
        crate::utils::run_command_output("git", &["branch", "--show-current"]).unwrap_or_default();

    let output =
        crate::utils::run_command_output("git", &["branch", "-v", "--sort=-committerdate"])
            .map_err(|e| format!("Failed to list branches: {}", e))?;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("* ") || trimmed.starts_with(&current) {
            println!(
                "{} {}",
                "▶".style(style::Theme::ACCENT),
                trimmed.style(style::Theme::ACCENT)
            );
        } else {
            println!("  {}", trimmed.style(style::Theme::MUTED));
        }
    }

    println!("{}", style::divider());
    Ok(())
}
