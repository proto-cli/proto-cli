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

fn should_color() -> bool {
    std::env::var("NO_COLOR").map_or(true, |v| v != "1" && !v.is_empty())
}

fn apply(text: &str, style: owo_colors::Style) -> String {
    if should_color() {
        text.style(style).to_string()
    } else {
        text.to_string()
    }
}

pub struct Spinner {
    spinner: indicatif::ProgressBar,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        let sp = if should_color() {
            indicatif::ProgressBar::new_spinner()
                .with_message(msg.to_string())
                .with_style(
                    indicatif::ProgressStyle::with_template("{spinner:.cyan} {msg}")
                        .unwrap()
                        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
                )
        } else {
            indicatif::ProgressBar::new_spinner()
                .with_message(msg.to_string())
                .with_style(indicatif::ProgressStyle::with_template("{spinner} {msg}").unwrap())
        };
        sp.enable_steady_tick(std::time::Duration::from_millis(80));
        Self { spinner: sp }
    }

    pub fn done(&self, msg: &str) {
        self.spinner.finish_with_message(msg.to_string());
    }

    pub fn fail(&self, msg: &str) {
        self.spinner.finish_with_message(format!(
            "{} {}",
            apply("✗", Theme::ERROR),
            apply(msg, Theme::ERROR)
        ));
    }
}

pub fn proto_banner() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        apply("    ⣀⡀", Theme::ACCENT),
        apply("⢠⣤⡀⣾⣿⣿⠀⣤⣤⡄", Theme::ACCENT),
        apply("⢿⣿⡇⠘⠛⠁⢸⣿⣿⠃", Theme::ACCENT),
        apply("⠈⣉⣤⣾⣿⣿⡆⠉⣴⣶⣶", Theme::ACCENT),
        apply("⣾⣿⣿⣿⣿⣿⣿⡀⠻⠟⠃", Theme::ACCENT),
        apply("⠙⠛⠻⢿⣿⣿⣿⡇", Theme::ACCENT),
        apply("    ⠈⠙⠋⠁", Theme::ACCENT),
    )
}

pub fn divider() -> String {
    apply(&"─".repeat(40), Theme::MUTED)
}

pub fn label_value(label: &str, value: &str) -> String {
    format!(
        "{} {}",
        apply(&format!("{:>14}:", label), Theme::LABEL),
        apply(value, Theme::VALUE)
    )
}

pub fn header(text: &str) -> String {
    format!(
        "{} {}",
        apply("◆", Theme::ACCENT),
        apply(text, Theme::HEADER)
    )
}

pub fn success(msg: &str) -> String {
    format!("{} {}", apply("✔", Theme::SUCCESS), msg)
}

pub fn warn(msg: &str) -> String {
    format!("{} {}", apply("⚠", Theme::WARN), msg)
}

pub fn error(msg: &str) -> String {
    format!("{} {}", apply("✗", Theme::ERROR), msg)
}

pub fn muted(msg: &str) -> String {
    apply(msg, Theme::MUTED)
}

pub fn section(title: &str) -> String {
    format!(
        "\n{}\n{}\n",
        apply(title, Theme::HEADER),
        apply(&"─".repeat(title.len()), Theme::MUTED),
    )
}
