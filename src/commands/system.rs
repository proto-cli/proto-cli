use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Subcommand, Debug, Clone)]
pub enum SystemAction {}

pub fn run() {
    use crate::utils;
    use owo_colors::OwoColorize;

    println!("{}", style::proto_banner());
    println!("{}\n", "System Information".style(style::Theme::HEADER));

    println!("{}", style::section("System"));
    println!("{}", style::label_value("OS", &utils::distro_name()));
    println!("{}", style::label_value("Kernel", &utils::get_kernel()));
    println!("{}", style::label_value("Arch", &utils::get_arch()));
    println!("{}", style::label_value("Uptime", &utils::get_uptime()));
    println!("{}", style::label_value("Shell", &utils::get_shell()));
    println!("{}", style::label_value("DE/WM", &utils::get_de_wm()));
    println!("{}", style::label_value("Terminal", &utils::get_terminal()));

    println!("{}", style::section("Hardware"));
    {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        if let Some(cpu) = sys.cpus().first() {
            println!(
                "{}",
                style::label_value(
                    "CPU",
                    &format!("{} ({} cores)", cpu.brand(), sys.cpus().len())
                )
            );
        }

        let total_ram = sys.total_memory();
        let used_ram = sys.used_memory();
        println!(
            "{}",
            style::label_value(
                "RAM",
                &format!("{} / {}", format_bytes(used_ram), format_bytes(total_ram))
            )
        );

        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();
        if total_swap > 0 {
            println!(
                "{}",
                style::label_value(
                    "Swap",
                    &format!("{} / {}", format_bytes(used_swap), format_bytes(total_swap))
                )
            );
        }
    }

    println!("{}", style::section("Storage"));
    {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        for disk in sysinfo::Disks::new_with_refreshed_list().iter() {
            let total = disk.total_space();
            let avail = disk.available_space();
            let used = total - avail;
            let pct = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let mount = disk.mount_point().to_string_lossy().to_string();

            let bar = usage_bar(pct);
            println!(
                "{}  {}  {:.1}%",
                format!("{:>14}:", mount).style(style::Theme::LABEL),
                format!("{} / {}", format_bytes(used), format_bytes(total))
                    .style(style::Theme::MUTED),
                pct
            );
            if !bar.is_empty() {
                println!("{}", bar);
            }
        }
    }

    println!("{}", style::section("Packages"));
    let pms = utils::detect_package_managers();
    if !pms.is_empty() && !pms.contains(&utils::PackageManager::Unknown) {
        for pm in &pms {
            if let Some(count) = utils::get_package_count(pm) {
                println!(
                    "{}",
                    style::label_value(&format!("{} pkgs", pm.name()), &count.to_string())
                );
            }
        }
    } else {
        println!(
            "{}",
            style::label_value("Packages", "Could not detect package manager")
        );
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

fn usage_bar(pct: f64) -> String {
    let width: usize = 30;
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let color_str = if pct > 90.0 {
        style::Theme::ERROR
    } else if pct > 70.0 {
        style::Theme::WARN
    } else {
        style::Theme::SUCCESS
    };

    format!(
        "            {}{}",
        "█".repeat(filled).style(color_str),
        "░".repeat(empty).dimmed()
    )
}
