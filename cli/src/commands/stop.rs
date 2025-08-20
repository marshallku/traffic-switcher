use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tracing::{error, info};

use crate::command::Command as CommandTrait;
use crate::context::Context;

pub struct StopCommand {
    pub pid_file: Option<PathBuf>,
    pub pid: Option<u32>,
}

#[async_trait]
impl CommandTrait for StopCommand {
    async fn execute(&self, _ctx: &Context) -> Result<()> {
        let pid = if let Some(pid) = self.pid {
            pid
        } else if let Some(pid_file) = &self.pid_file {
            let pid_str = std::fs::read_to_string(pid_file)?;
            pid_str.trim().parse::<u32>()?
        } else {
            anyhow::bail!("Either --pid or --pid-file must be specified");
        };

        info!("Stopping traffic-switcher process with PID {}...", pid);

        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            match signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    println!("Sent SIGTERM to process {}", pid);
                    
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    
                    match signal::kill(Pid::from_raw(pid as i32), None) {
                        Ok(_) => {
                            info!("Process {} is still running, sending SIGKILL", pid);
                            signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)?;
                            println!("Process {} forcefully terminated", pid);
                        }
                        Err(_) => {
                            println!("Process {} stopped successfully", pid);
                        }
                    }
                    
                    if let Some(pid_file) = &self.pid_file {
                        std::fs::remove_file(pid_file).ok();
                    }
                }
                Err(e) => {
                    error!("Failed to stop process {}: {}", pid, e);
                    anyhow::bail!("Failed to stop process: {}", e);
                }
            }
        }

        #[cfg(not(unix))]
        {
            anyhow::bail!("Stop command is only supported on Unix systems");
        }

        Ok(())
    }
}