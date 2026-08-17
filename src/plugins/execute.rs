use std::process::{Command, Stdio};

pub fn execute_plugin(binary_path: &std::path::Path, args: &[String]) -> Result<(), String> {
    let status = Command::new(binary_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to execute plugin: {}", e))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
