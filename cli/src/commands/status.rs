use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::command::Command as CommandTrait;
use crate::context::Context;

pub struct StatusCommand {
    pub pid_file: Option<PathBuf>,
}

#[async_trait]
impl CommandTrait for StatusCommand {
    async fn execute(&self, ctx: &Context) -> Result<()> {
        let mut pid_running = false;
        let mut pid_value = None;
        
        if let Some(pid_file) = &self.pid_file {
            if let Ok(pid_str) = std::fs::read_to_string(pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    pid_value = Some(pid);
                    
                    #[cfg(unix)]
                    {
                        use nix::sys::signal;
                        use nix::unistd::Pid;
                        
                        if signal::kill(Pid::from_raw(pid as i32), None).is_ok() {
                            pid_running = true;
                        }
                    }
                }
            }
        }
        
        let api_running = ctx.client
            .get(&format!("{}/", ctx.api_url))
            .send()
            .await
            .is_ok();
        
        println!("Traffic Switcher Status:");
        println!("------------------------");
        
        if let Some(pid) = pid_value {
            if pid_running {
                println!("✓ Process: Running (PID: {})", pid);
            } else {
                println!("✗ Process: Not running (stale PID: {})", pid);
            }
        } else {
            println!("✗ Process: No PID file found");
        }
        
        if api_running {
            println!("✓ API Server: Running at {}", ctx.api_url);
            
            if let Ok(response) = ctx.client.get(&format!("{}/config", ctx.api_url)).send().await {
                if let Ok(config) = response.json::<serde_json::Value>().await {
                    if let Some(proxy_port) = config.get("proxy_port") {
                        println!("✓ Proxy Server: Port {}", proxy_port);
                    }
                    if let Some(services) = config.get("services").and_then(|s| s.as_object()) {
                        println!("\nActive Services ({}):", services.len());
                        for (name, service) in services {
                            if let Some(port) = service.get("port") {
                                println!("  - {}: port {}", name, port);
                            }
                        }
                    }
                }
            }
        } else {
            println!("✗ API Server: Not responding at {}", ctx.api_url);
        }
        
        Ok(())
    }
}