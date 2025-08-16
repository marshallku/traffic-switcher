use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;

#[derive(Parser)]
#[command(name = "traffic-switcher")]
#[command(about = "A CLI tool for managing traffic switching")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a specific route
    Test {
        /// Route to test
        route: String,

        /// Target URL
        target: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Test { route, target } => {
            println!("Testing route: {} -> {}", route, target);
        }
    }

    Ok(())
}
