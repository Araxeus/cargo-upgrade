use std::io::Result;

use cargo_upgrade_command::{show_outdated_packages, update_all_packages};

fn main() -> Result<()> {
    let arg = std::env::args().next_back().unwrap_or_else(|| {
        print_help();
        std::process::exit(1);
    });
    run_command(arg.trim_start_matches('-'))?;

    Ok(())
}

fn run_command(cmd: &str) -> Result<()> {
    match cmd {
        "update" | "upgrade" | "u" => update_all_packages(),
        "outdated" | "list" | "show" | "o" | "l" => show_outdated_packages(),
        "version" | "v" => {
            print_version();
            Ok(())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    print_version();
    println!("Usage: cargo upgrade [command]");
    println!("Commands:");
    println!("  -h, --help\t\t\tPrint this help message");
    println!("  -u, --update, --upgrade\tUpdate all outdated crates");
    println!("  -o, --outdated, --list\tShow all outdated crates");
    println!("  -v, --version\t\t\tPrint the version");
}
