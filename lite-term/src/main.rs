use anyhow::Result;
use lite_term::Application;
use std::env;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
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

    // Open file if provided as argument
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        for path in &args[1..] {
            if let Err(e) = app.open(path) {
                eprintln!("Error opening {}: {}", path, e);
            }
        }
    }

    // Run the application
    app.run().await?;

    Ok(())
}
