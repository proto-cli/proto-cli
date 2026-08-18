use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum PkgAction {
    #[command(about = "Install one or more packages")]
    Install {
        #[arg(required = true, num_args = 1.., value_name = "PACKAGE")]
        packages: Vec<String>,
    },
    #[command(about = "Search for a package")]
    Search {
        #[arg(required = true, value_name = "QUERY")]
        query: String,
    },
    #[command(about = "Remove one or more packages")]
    Remove {
        #[arg(required = true, num_args = 1.., value_name = "PACKAGE")]
        packages: Vec<String>,
    },
    #[command(about = "Update all or specific packages")]
    Update {
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },
    #[command(about = "List installed packages")]
    List,
}

pub fn run(action: &PkgAction) {
    use crate::style;
    use crate::utils::{self, PackageManager};

    let pm = utils::default_package_manager();
    if pm == PackageManager::Unknown {
        eprintln!("{}", style::error("No supported package manager detected."));
        eprintln!(
            "{} Supported: pacman, yay, paru, apt, dnf, zypper, apk",
            style::warn("")
        );
        return;
    }

    let spinner = style::Spinner::new(&format!("Using {}", pm.name()));
    std::thread::sleep(std::time::Duration::from_millis(300));
    spinner.done(&format!("Using {}", pm.name()));

    let result = match action {
        PkgAction::Install { packages } => install(&pm, packages),
        PkgAction::Search { query } => search(&pm, query),
        PkgAction::Remove { packages } => remove(&pm, packages),
        PkgAction::Update { package } => update(&pm, package.as_deref()),
        PkgAction::List => list(&pm),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style::error(""), e);
        std::process::exit(1);
    }
}

fn install(pm: &crate::utils::PackageManager, packages: &[String]) -> Result<(), String> {
    let args = build_install_args(pm, packages);
    let status = crate::utils::run_command(pm.name(), &args)
        .map_err(|e| format!("Failed to run {}: {}", pm.name(), e))?;
    if !status.success() {
        return Err(format!("{} exited with error", pm.name()));
    }
    Ok(())
}

fn search(pm: &crate::utils::PackageManager, query: &str) -> Result<(), String> {
    let args = build_search_args(pm, query);
    let status = crate::utils::run_command(pm.name(), &args)
        .map_err(|e| format!("Failed to run {}: {}", pm.name(), e))?;
    if !status.success() {
        return Err(format!("{} exited with error", pm.name()));
    }
    Ok(())
}

fn remove(pm: &crate::utils::PackageManager, packages: &[String]) -> Result<(), String> {
    let args = build_remove_args(pm, packages);
    let status = crate::utils::run_command(pm.name(), &args)
        .map_err(|e| format!("Failed to run {}: {}", pm.name(), e))?;
    if !status.success() {
        return Err(format!("{} exited with error", pm.name()));
    }
    Ok(())
}

fn update(pm: &crate::utils::PackageManager, package: Option<&str>) -> Result<(), String> {
    let (cmd, args) = build_update_args(pm, package);
    let status = crate::utils::run_command(cmd, &args)
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;
    if !status.success() {
        return Err(format!("{} exited with error", cmd));
    }
    Ok(())
}

fn list(pm: &crate::utils::PackageManager) -> Result<(), String> {
    let (cmd, args) = build_list_args(pm);
    let status = crate::utils::run_command(cmd, &args)
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;
    if !status.success() {
        return Err(format!("{} exited with error", cmd));
    }
    Ok(())
}

fn build_install_args<'a>(
    pm: &crate::utils::PackageManager,
    packages: &'a [String],
) -> Vec<&'a str> {
    match pm {
        crate::utils::PackageManager::Pacman => {
            let mut v = vec!["-S", "--noconfirm"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Yay | crate::utils::PackageManager::Paru => {
            let mut v = vec!["-S", "--noconfirm"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Apt => {
            let mut v = vec!["install", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Dnf => {
            let mut v = vec!["install", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Zypper => {
            let mut v = vec!["install", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Apk => {
            let mut v = vec!["add"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Unknown => vec![],
    }
}

fn build_search_args<'a>(pm: &crate::utils::PackageManager, query: &'a str) -> Vec<&'a str> {
    match pm {
        crate::utils::PackageManager::Pacman => vec!["-Ss", query],
        crate::utils::PackageManager::Yay => vec!["-Ss", query],
        crate::utils::PackageManager::Paru => vec!["-Ss", query],
        crate::utils::PackageManager::Apt => vec!["search", query],
        crate::utils::PackageManager::Dnf => vec!["search", query],
        crate::utils::PackageManager::Zypper => vec!["search", query],
        crate::utils::PackageManager::Apk => vec!["search", query],
        crate::utils::PackageManager::Unknown => vec![],
    }
}

fn build_remove_args<'a>(
    pm: &crate::utils::PackageManager,
    packages: &'a [String],
) -> Vec<&'a str> {
    match pm {
        crate::utils::PackageManager::Pacman => {
            let mut v = vec!["-R", "--noconfirm"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Yay | crate::utils::PackageManager::Paru => {
            let mut v = vec!["-R", "--noconfirm"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Apt => {
            let mut v = vec!["remove", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Dnf => {
            let mut v = vec!["remove", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Zypper => {
            let mut v = vec!["remove", "-y"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Apk => {
            let mut v = vec!["del"];
            v.extend(packages.iter().map(|s| s.as_str()));
            v
        }
        crate::utils::PackageManager::Unknown => vec![],
    }
}

fn build_update_args<'a>(
    pm: &crate::utils::PackageManager,
    package: Option<&'a str>,
) -> (&'a str, Vec<&'a str>) {
    match pm {
        crate::utils::PackageManager::Pacman => {
            if let Some(pkg) = package {
                ("pacman", vec!["-S", "--noconfirm", pkg])
            } else {
                ("pacman", vec!["-Syu", "--noconfirm"])
            }
        }
        crate::utils::PackageManager::Yay => {
            if let Some(pkg) = package {
                ("yay", vec!["-S", "--noconfirm", pkg])
            } else {
                ("yay", vec!["-Syu", "--noconfirm"])
            }
        }
        crate::utils::PackageManager::Paru => {
            if let Some(pkg) = package {
                ("paru", vec!["-S", "--noconfirm", pkg])
            } else {
                ("paru", vec!["-Syu", "--noconfirm"])
            }
        }
        crate::utils::PackageManager::Apt => {
            if let Some(pkg) = package {
                ("apt", vec!["install", "--only-upgrade", "-y", pkg])
            } else {
                ("apt", vec!["update"])
            }
        }
        crate::utils::PackageManager::Dnf => {
            if let Some(pkg) = package {
                ("dnf", vec!["upgrade", "-y", pkg])
            } else {
                ("dnf", vec!["upgrade", "-y"])
            }
        }
        crate::utils::PackageManager::Zypper => {
            if let Some(pkg) = package {
                ("zypper", vec!["update", "-y", pkg])
            } else {
                ("zypper", vec!["update", "-y"])
            }
        }
        crate::utils::PackageManager::Apk => {
            if let Some(pkg) = package {
                ("apk", vec!["upgrade", pkg])
            } else {
                ("apk", vec!["upgrade"])
            }
        }
        crate::utils::PackageManager::Unknown => ("echo", vec!["No package manager"]),
    }
}

fn build_list_args<'a>(pm: &crate::utils::PackageManager) -> (&'a str, Vec<&'a str>) {
    match pm {
        crate::utils::PackageManager::Pacman => ("pacman", vec!["-Q"]),
        crate::utils::PackageManager::Yay => ("yay", vec!["-Q"]),
        crate::utils::PackageManager::Paru => ("paru", vec!["-Q"]),
        crate::utils::PackageManager::Apt => {
            ("dpkg-query", vec!["-W", "-f", "${Package} ${Version}\n"])
        }
        crate::utils::PackageManager::Dnf => ("rpm", vec!["-qa"]),
        crate::utils::PackageManager::Zypper => ("rpm", vec!["-qa"]),
        crate::utils::PackageManager::Apk => ("apk", vec!["info", "-v"]),
        crate::utils::PackageManager::Unknown => ("echo", vec!["No package manager"]),
    }
}
