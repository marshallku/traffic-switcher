use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::{error, info};

use crate::command::Command as CommandTrait;
use crate::context::Context;

pub struct StartCommand {
    pub config: Option<PathBuf>,
    pub daemon: bool,
    pub log_file: Option<PathBuf>,
    pub pid_file: Option<PathBuf>,
    pub verbose: bool,
}

#[async_trait]
impl CommandTrait for StartCommand {
    async fn execute(&self, _ctx: &Context) -> Result<()> {
        let mut cmd = Command::new("traffic_switcher");

        if let Some(config) = &self.config {
            cmd.env("CONFIG_PATH", config);
        }

        if self.verbose {
            cmd.env("RUST_LOG", "debug");
        } else {
            cmd.env("RUST_LOG", "info");
        }

        if self.daemon {
            info!("Starting traffic-switcher in daemon mode...");
            
            cmd.stdin(Stdio::null());
            
            if let Some(log_file) = &self.log_file {
                let log = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)?;
                cmd.stdout(log.try_clone()?);
                cmd.stderr(log);
            } else {
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
            }

            let child = cmd.spawn()?;
            let pid = child.id();
            
            if let Some(pid_file) = &self.pid_file {
                std::fs::write(pid_file, pid.to_string())?;
                info!("Process started with PID {} (written to {:?})", pid, pid_file);
            } else {
                info!("Process started with PID {}", pid);
            }
            
            println!("Traffic Switcher started in background (PID: {})", pid);
            if let Some(log_file) = &self.log_file {
                println!("Logs are being written to: {}", log_file.display());
            }
        } else {
            info!("Starting traffic-switcher in foreground...");
            
            let status = cmd.status()?;
            
            if !status.success() {
                error!("Traffic Switcher exited with status: {}", status);
                anyhow::bail!("Traffic Switcher failed to start");
            }
        }

        Ok(())
    }
}