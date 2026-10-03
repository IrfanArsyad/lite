use anyhow::Result;
use lite_config::Config;
use lite_term::{restore_terminal, Application, Event};
use std::env;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "IrfanArsyad/lite";
/// Seconds to wait for GitHub before giving up on the background update check
const UPDATE_CHECK_TIMEOUT_SECS: &str = "2";

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
    println!("    lite file.txt         Open file (created on first save if missing)");
    println!("    lite a.rs b.rs        Open multiple files");
    println!("    lite --update         Update lite to latest");
    if let Some(path) = Config::default_path() {
        println!();
        println!("CONFIG:");
        println!("    {}", path.display());
    }
}

fn install_url() -> String {
    format!("https://raw.githubusercontent.com/{REPO}/main/scripts/install.sh")
}

/// Latest released version on GitHub, without the leading `v`
fn fetch_latest_version(timeout_secs: Option<&str>) -> Option<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let mut cmd = Command::new("curl");
    cmd.arg("-fsSL");
    if let Some(secs) = timeout_secs {
        cmd.args(["--connect-timeout", secs, "--max-time", secs]);
    }
    let output = cmd.arg(url).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_tag_name(&String::from_utf8_lossy(&output.stdout))
}

/// Pull `tag_name` out of the GitHub release JSON without a JSON parser
fn parse_tag_name(json: &str) -> Option<String> {
    let rest = &json[json.find("\"tag_name\"")?..];
    let after_colon = rest[rest.find(':')? + 1..].trim_start().strip_prefix('"')?;
    let tag = &after_colon[..after_colon.find('"')?];
    Some(tag.trim_start_matches('v').to_string())
}

/// Whether `candidate` is a newer `major.minor.patch` version than `current`
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parse(v: &str) -> Option<Vec<u64>> {
        v.split('-')
            .next()?
            .split('.')
            .map(|part| part.parse().ok())
            .collect()
    }
    match (parse(candidate), parse(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn run_install_script() -> bool {
    Command::new("bash")
        .args(["-c", &format!("curl -fsSL {} | bash", install_url())])
        .status()
        .is_ok_and(|s| s.success())
}

fn update() -> Result<()> {
    println!("Checking for updates...");

    match fetch_latest_version(None) {
        Some(latest) if is_newer(&latest, VERSION) => {
            println!("New version available: v{latest} (current: v{VERSION})");
            println!("Updating...");
            println!();
            if run_install_script() {
                println!();
                println!("Update complete! Restart lite to use the new version.");
            } else {
                eprintln!("Update failed. Try manually:");
                eprintln!("  curl -fsSL {} | bash", install_url());
            }
        }
        Some(_) => println!("Already up to date (v{VERSION})"),
        None => {
            eprintln!("Could not reach GitHub to check for updates.");
            eprintln!("To reinstall manually:");
            eprintln!("  curl -fsSL {} | bash", install_url());
        }
    }

    Ok(())
}

/// Restore the terminal before the panic message is printed; with
/// `panic = "abort"` no destructor would get the chance to.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("-h" | "--help") => {
            print_help();
            return Ok(());
        }
        Some("-v" | "--version") => {
            println!("lite {VERSION}");
            return Ok(());
        }
        Some("-u" | "--update") => return update(),
        _ => {}
    }

    let (config, config_error) = match Config::load() {
        Ok(config) => (config, None),
        Err(err) => (Config::default(), Some(err)),
    };

    install_panic_hook();
    let mut app = Application::new(config)?;

    for path in args.iter().filter(|a| !a.starts_with('-')) {
        if let Err(e) = app.open(path) {
            app.set_notice(format!("Error opening {path}: {e}"));
        }
    }
    if let Some(err) = config_error {
        app.set_notice(format!("{err}. Using default settings."));
    }

    let sender = app.event_sender();
    std::thread::spawn(move || {
        if let Some(latest) = fetch_latest_version(Some(UPDATE_CHECK_TIMEOUT_SECS)) {
            if is_newer(&latest, VERSION) {
                let _ = sender.send(Event::Notice(format!(
                    "Update available: v{VERSION} -> v{latest}. Run 'lite --update' to update."
                )));
            }
        }
    });

    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    runtime.block_on(app.run())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_name() {
        let json = r#"{"url":"x","tag_name": "v1.2.3","name":"Release"}"#;
        assert_eq!(parse_tag_name(json), Some("1.2.3".to_string()));
        assert_eq!(parse_tag_name("{}"), None);
    }

    #[test]
    fn test_is_newer_compares_numerically() {
        assert!(is_newer("0.10.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
        assert!(!is_newer("garbage", "0.1.0"));
    }
}
