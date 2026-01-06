use anyhow::Result;
use lite_term::Application;
use std::env;
use std::process::Command;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "IrfanArsyad/lite";

fn print_help() {
    println!("lite - A lightweight terminal text editor");
    println!();
    println!("USAGE:");
    println!("    lite [OPTIONS] [FILES]...");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -v, --version    Print version information");
    println!("    -u, --update     Update to latest version");
    println!();
    println!("EXAMPLES:");
    println!("    lite                  Open new buffer");
    println!("    lite file.txt         Open file");
    println!("    lite a.rs b.rs        Open multiple files");
    println!("    lite --update         Update lite to latest");
}

fn print_version() {
    println!("lite {}", VERSION);
}

fn check_for_update() -> Option<String> {
    let output = Command::new("curl")
        .args(["-sL", "--connect-timeout", "2", &format!("https://api.github.com/repos/{}/releases/latest", REPO)])
        .output()
        .ok()?;

    let json = String::from_utf8_lossy(&output.stdout);
    if let Some(start) = json.find("\"tag_name\"") {
        let rest = &json[start..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..];
            let trimmed = after_colon.trim().trim_start_matches('"');
            if let Some(end) = trimmed.find('"') {
                let latest = trimmed[..end].trim_start_matches('v').to_string();
                if latest != VERSION {
                    return Some(latest);
                }
            }
        }
    }
    None
}

fn update() -> Result<()> {
    println!("Checking for updates...");

    // Check latest version from GitHub API
    let output = Command::new("curl")
        .args(["-sL", &format!("https://api.github.com/repos/{}/releases/latest", REPO)])
        .output();

    let latest_version = match output {
        Ok(out) => {
            let json = String::from_utf8_lossy(&out.stdout);
            // Simple parse for tag_name
            if let Some(start) = json.find("\"tag_name\"") {
                let rest = &json[start..];
                if let Some(colon) = rest.find(':') {
                    let after_colon = &rest[colon + 1..];
                    let trimmed = after_colon.trim().trim_start_matches('"');
                    if let Some(end) = trimmed.find('"') {
                        Some(trimmed[..end].trim_start_matches('v').to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };

    match latest_version {
        Some(latest) if latest != VERSION => {
            println!("New version available: v{} (current: v{})", latest, VERSION);
            println!("Updating...");
            println!();

            // Run install script
            let status = Command::new("bash")
                .args(["-c", &format!(
                    "curl -sL https://raw.githubusercontent.com/{}/main/scripts/install.sh | bash",
                    REPO
                )])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!();
                    println!("Update complete! Restart lite to use the new version.");
                }
                _ => {
                    eprintln!("Update failed. Try manually:");
                    eprintln!("  curl -fsSL https://raw.githubusercontent.com/{}/main/scripts/install.sh | bash", REPO);
                }
            }
        }
        Some(_) => {
            println!("Already up to date (v{})", VERSION);
        }
        None => {
            println!("Could not check for updates. Reinstalling...");
            let _ = Command::new("bash")
                .args(["-c", &format!(
                    "curl -sL https://raw.githubusercontent.com/{}/main/scripts/install.sh | bash",
                    REPO
                )])
                .status();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle CLI flags
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            "-u" | "--update" => {
                return update();
            }
            _ => {}
        }
    }

    // Setup logging (to file to avoid messing with TUI)
    let log_file = std::fs::File::create("/tmp/lite.log").ok();
    if let Some(file) = log_file {
        tracing_subscriber::registry()
            .with(fmt::layer().with_writer(file))
            .with(EnvFilter::from_default_env().add_directive("lite=debug".parse()?))
            .init();
    }

    // Create application
    let mut app = Application::new()?;

    // Open files if provided as arguments
    for path in args.iter().skip(1) {
        if !path.starts_with('-') {
            if let Err(e) = app.open(path) {
                eprintln!("Error opening {}: {}", path, e);
            }
        }
    }

    // Check for updates in background (non-blocking)
    std::thread::spawn(|| {
        if let Some(new_version) = check_for_update() {
            // Write to a temp file that we can read later
            let _ = std::fs::write("/tmp/lite_update_available", new_version);
        }
    });

    // Check if update notification exists from previous check
    if let Ok(new_version) = std::fs::read_to_string("/tmp/lite_update_available") {
        app.set_update_notice(format!(
            "Update available: v{} -> v{}. Run 'lite --update' to update.",
            VERSION, new_version.trim()
        ));
        let _ = std::fs::remove_file("/tmp/lite_update_available");
    }

    // Run the application
    app.run().await?;

    Ok(())
}
