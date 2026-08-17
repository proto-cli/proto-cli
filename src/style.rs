use owo_colors::OwoColorize;

pub struct Theme;

impl Theme {
    pub const HEADER: owo_colors::Style = owo_colors::Style::new().bold().bright_blue();
    pub const ACCENT: owo_colors::Style = owo_colors::Style::new().bold().cyan();
    pub const MUTED: owo_colors::Style = owo_colors::Style::new().dimmed();
    pub const SUCCESS: owo_colors::Style = owo_colors::Style::new().bright_green();
    pub const WARN: owo_colors::Style = owo_colors::Style::new().bright_yellow();
    pub const ERROR: owo_colors::Style = owo_colors::Style::new().bright_red();
    pub const BOLD: owo_colors::Style = owo_colors::Style::new().bold();
    pub const LABEL: owo_colors::Style = owo_colors::Style::new().bright_cyan();
    pub const VALUE: owo_colors::Style = owo_colors::Style::new().bright_white();
}

pub struct Spinner {
    spinner: indicatif::ProgressBar,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        let sp = indicatif::ProgressBar::new_spinner()
            .with_message(msg.to_string())
            .with_style(
                indicatif::ProgressStyle::with_template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
        sp.enable_steady_tick(std::time::Duration::from_millis(80));
        Self { spinner: sp }
    }

    pub fn done(&self, msg: &str) {
        self.spinner.finish_with_message(msg.to_string());
    }

    pub fn fail(&self, msg: &str) {
        self.spinner.finish_with_message(
            format!("{} {}", "✗".style(Theme::ERROR), msg.style(Theme::ERROR))
        );
    }
}

pub fn proto_banner() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        "    ⣀⡀".cyan(),
        "⢠⣤⡀⣾⣿⣿⠀⣤⣤⡄".cyan(),
        "⢿⣿⡇⠘⠛⠁⢸⣿⣿⠃".cyan(),
        "⠈⣉⣤⣾⣿⣿⡆⠉⣴⣶⣶".cyan(),
        "⣾⣿⣿⣿⣿⣿⣿⡀⠻⠟⠃".cyan(),
        "⠙⠛⠻⢿⣿⣿⣿⡇".cyan(),
        "    ⠈⠙⠋⠁".cyan(),
    )
}

pub fn divider() -> String {
    "─".repeat(40).dimmed().to_string()
}

pub fn label_value(label: &str, value: &str) -> String {
    format!(
        "{} {}",
        format!("{:>14}:", label).style(Theme::LABEL),
        value.style(Theme::VALUE)
    )
}

pub fn header(text: &str) -> String {
    format!("{} {}", "◆".style(Theme::ACCENT), text.style(Theme::HEADER))
}

pub fn success(msg: &str) -> String {
    format!("{} {}", "✔".style(Theme::SUCCESS), msg)
}

pub fn warn(msg: &str) -> String {
    format!("{} {}", "⚠".style(Theme::WARN), msg)
}

pub fn error(msg: &str) -> String {
    format!("{} {}", "✗".style(Theme::ERROR), msg)
}

pub fn muted(msg: &str) -> String {
    format!("{}", msg.style(Theme::MUTED))
}

pub fn section(title: &str) -> String {
    format!(
        "\n{}\n{}\n",
        title.style(Theme::HEADER).bold().to_string(),
        "─".repeat(title.len()).dimmed().to_string()
    )
}
