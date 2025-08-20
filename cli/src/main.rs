use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod command;
mod commands;
mod context;

#[derive(Parser, Clone)]
#[command(name = "tsctl")]
#[command(about = "Traffic Switcher CLI - Port-based deployment tool", long_about = None)]
struct Cli {
    /// API server URL
    #[arg(short, long, default_value = "http://localhost:1143")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Start the traffic-switcher server
    Start {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Run in daemon mode (background)
        #[arg(short, long)]
        daemon: bool,
        /// Log file path (for daemon mode)
        #[arg(short, long)]
        log_file: Option<PathBuf>,
        /// PID file path (for daemon mode)
        #[arg(short, long)]
        pid_file: Option<PathBuf>,
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Stop the traffic-switcher server
    Stop {
        /// PID file path
        #[arg(short = 'f', long)]
        pid_file: Option<PathBuf>,
        /// Process ID to stop
        #[arg(short, long)]
        pid: Option<u32>,
    },
    /// Check server status
    Status {
        /// PID file path
        #[arg(short, long)]
        pid_file: Option<PathBuf>,
    },
    /// Update a service to use a different port
    Port {
        /// Service name
        service: String,
        /// New port number
        port: u16,
        /// Skip health check
        #[arg(short, long, default_value = "false")]
        skip_health: bool,
    },
    /// Switch a service between two ports (blue-green deployment)
    Switch {
        /// Service name
        service: String,
        /// Current port (will be verified)
        from: u16,
        /// Target port
        to: u16,
        /// Skip health check
        #[arg(short, long)]
        skip_health: bool,
    },
    /// Rollback a service to its previous port
    Rollback {
        /// Service name
        service: String,
    },
    /// List all services with their current ports
    Services,
    /// List all routes
    Routes,
    /// Check health of a service
    Health {
        /// Service name
        service: String,
    },
    /// Show current configuration
    Config,
    /// Perform automated blue-green deployment
    Deploy {
        /// Service name
        service: String,
        /// Previous port (current instance)
        previous_port: u16,
        /// Next port (new instance)
        next_port: u16,
        /// Skip health check
        #[arg(short, long)]
        skip_health: bool,
    },
    /// Show current port for a service
    Current {
        /// Service name
        service: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let ctx = context::Context::new(cli.api_url.clone());

    use command::Command;
    use commands::port::PortCommand;
    use commands::start::StartCommand;
    use commands::stop::StopCommand;
    use commands::status::StatusCommand;

    match &cli.command {
        Commands::Start {
            config,
            daemon,
            log_file,
            pid_file,
            verbose,
        } => {
            let cmd = StartCommand {
                config: config.clone(),
                daemon: *daemon,
                log_file: log_file.clone(),
                pid_file: pid_file.clone(),
                verbose: *verbose,
            };
            cmd.execute(&ctx).await?;
        }
        Commands::Stop { pid_file, pid } => {
            let cmd = StopCommand {
                pid_file: pid_file.clone(),
                pid: *pid,
            };
            cmd.execute(&ctx).await?;
        }
        Commands::Status { pid_file } => {
            let cmd = StatusCommand {
                pid_file: pid_file.clone(),
            };
            cmd.execute(&ctx).await?;
        }
        Commands::Port {
            service,
            port,
            skip_health,
        } => {
            let cmd = PortCommand {
                service: service.clone(),
                port: *port,
                skip_health: *skip_health,
            };
            cmd.execute(&ctx).await?;
        }
        Commands::Switch {
            service,
            from,
            to,
            skip_health,
        } => {
            println!(
                "Switch: {} -> {} (from: {}, skip_health: {})",
                service, to, from, skip_health
            );
        }
        Commands::Rollback { service } => {
            println!("Rollback: {}", service);
        }
        Commands::Services => {
            println!("Services");
        }
        Commands::Routes => {
            println!("Routes");
        }
        Commands::Health { service } => {
            println!("Health: {}", service);
        }
        Commands::Config => {
            println!("Config");
        }
        Commands::Deploy {
            service,
            previous_port,
            next_port,
            skip_health,
        } => {
            println!(
                "Deploy: {} (previous: {}, next: {}, skip_health: {})",
                service, previous_port, next_port, skip_health
            );
        }
        Commands::Current { service } => {
            println!("Current: {}", service);
        }
    }

    Ok(())
}
