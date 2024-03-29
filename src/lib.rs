use std::env;
use std::fs::{create_dir, remove_dir_all, rename};
use std::io::{BufRead, BufReader, Result};

use std::process::{Command, Stdio};
use std::time::Instant;

use colored::Colorize;
use spinoff::{spinners, Color, Spinner};

pub struct Package {
    name: String,
    version: String,
    new_version: Option<String>,
}

impl Package {
    fn to_formatted(&self) -> Self {
        let name = self.name.to_string();
        let old_version = format!("v{}", self.version).bright_red();
        let new_version = format!("v{}", self.new_version.as_ref().unwrap()).bright_green();

        Self {
            name,
            version: old_version.to_string(),
            new_version: Some(new_version.to_string()),
        }
    }
}

/// # Errors
/// Will return `Err` if the command fails to execute
///
/// # Panics
/// Will panic if the command fails to execute
pub fn get_installed_packages() -> Result<Vec<Package>> {
    let output = Command::new("cargo").args(["install", "--list"]).output()?;

    let text = String::from_utf8_lossy(&output.stdout);

    let mut packages = Vec::new();
    for line in text.lines() {
        if line.ends_with(':') {
            let parts: Vec<_> = line.splitn(2, ' ').collect();
            if parts.len() == 2 && parts[1].starts_with('v') {
                let name = parts[0].trim().to_string();
                let version = parts[1]
                    .trim()
                    .trim_end_matches(':')
                    .trim_start_matches('v')
                    .to_string();
                packages.push(Package {
                    name,
                    version,
                    new_version: None,
                });
            }
        }
    }

    Ok(packages)
}

/// # Errors
/// Will return `Err` if the command fails to execute
///
/// # Panics
/// Will panic if the command fails to execute
pub fn get_outdated_packages() -> Result<Vec<Package>> {
    let spinner = Spinner::new(
        spinners::Dots,
        "Scanning for outdated crates...",
        Color::Cyan,
    );

    let packages = get_installed_packages()?;

    let mut outdated_packages = Vec::new();

    for package in &packages {
        let output = Command::new("cargo")
            .args(["search", &package.name, "--limit=1", "--color=never", "-q"])
            .output()?;
        let text = String::from_utf8_lossy(&output.stdout);

        let prefix = format!("{} = \"", package.name);

        if !text.starts_with(&prefix) {
            continue;
        }

        let value_start = prefix.len();
        let quote_end = text[value_start..].find('"').unwrap();
        let latest_version = text[value_start..value_start + quote_end].to_string();

        if latest_version != package.version {
            outdated_packages.push(Package {
                name: package.name.to_string(),
                version: package.version.clone(),
                new_version: Some(latest_version),
            });
        }
    }

    spinner.clear();

    Ok(outdated_packages)
}

/// # Errors
/// Will return `Err` if the command fails to execute
///
/// # Panics
/// Will panic if the command fails to execute
pub fn show_outdated_packages() -> Result<()> {
    let outdated_packages = get_outdated_packages()?;
    if outdated_packages.is_empty() {
        return Ok(());
    }
    println!("Outdated global cargo crates:");
    println!("===============================");
    for package in outdated_packages {
        let formatted = package.to_formatted();

        println!(
            "📦 {}: {} -> {}",
            formatted.name,
            formatted.version,
            formatted.new_version.unwrap()
        );
    }

    Ok(())
}

/// # Errors
///
/// Will return `Err` if the command fails to execute
///
/// # Panics
/// Will panic if the command fails to execute
pub fn update_package(name: &str) -> Result<()> {
    let mut spinner = Spinner::new(spinners::Dots, "Loading...", Color::Cyan);

    let start_time = Instant::now();

    let mut cmd = Command::new("cargo")
        .args(["install", name, "--locked"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;

    let reader = BufReader::new(cmd.stderr.take().unwrap());

    let mut last_line = String::new();

    for line in reader.lines() {
        last_line = line?;
        spinner.update_text(last_line.trim().to_string());
    }

    let status_code = cmd.wait()?;

    let status = format!("{} [{:.2?}]", last_line.trim(), start_time.elapsed());

    match status_code.code().unwrap_or(1) {
        0 => spinner.success(&status),
        1 => spinner.fail(&status),
        _ => spinner.warn(&status),
    }

    Ok(())
}

/// # Errors
/// Will return `Err` if the command fails to execute
///
/// # Panics
/// Will panic if the command fails to execute
pub fn update_all_packages() -> Result<()> {
    let packages = get_outdated_packages()?;

    let mut done_one = false;

    for package in packages {
        if package.new_version.is_none() {
            continue;
        }
        let formatted = package.to_formatted();
        if done_one {
            println!();
        }
        if package.name == env!("CARGO_PKG_NAME") {
            if cfg!(debug_assertions) {
                println!("Skipping self update in debug mode");
                continue;
            }
            move_executable_to_temp_folder()?;
        }
        println!(
            "Upgrading {} from {} to {}",
            formatted.name,
            formatted.version,
            formatted.new_version.unwrap()
        );
        update_package(&package.name)?;
        done_one = true;
    }

    Ok(())
}

fn move_executable_to_temp_folder() -> Result<()> {
    let current_exe = env::current_exe()?;
    let temp_dir = env::temp_dir();

    // Generate a unique file name for the executable in the temp directory
    let cloned_exe_dir = temp_dir.join(env!("CARGO_PKG_NAME"));
    if cloned_exe_dir.exists() {
        remove_dir_all(&cloned_exe_dir)?;
    }

    create_dir(&cloned_exe_dir)?;

    let mut cloned_exe_path = cloned_exe_dir.join(current_exe.file_name().unwrap());
    let mut i = 0;
    while cloned_exe_path.exists() {
        i += 1;

        cloned_exe_path = cloned_exe_dir.join(format!(
            "{}-{i}",
            current_exe.file_stem().unwrap().to_str().unwrap()
        ));
        if cfg!(windows) {
            cloned_exe_path.set_extension("exe");
        }
    }

    // Move the current executable to the temp directory
    rename(&current_exe, &cloned_exe_path)?;

    Ok(())
}
