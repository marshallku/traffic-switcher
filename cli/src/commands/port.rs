use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use serde_json::json;

use crate::command::Command;
use crate::context::Context;

pub struct PortCommand {
    pub service: String,
    pub port: u16,
    pub skip_health: bool,
}

#[async_trait]
impl Command for PortCommand {
    async fn execute(&self, ctx: &Context) -> Result<()> {
        println!(
            "{}",
            format!("Updating {} to port {}...", self.service, self.port).blue()
        );

        println!(
            "{}",
            format!(
                "Skipping health check: {}",
                if self.skip_health { "yes" } else { "no" }
            )
            .blue()
        );

        let response = ctx
            .client
            .post(ctx.api_endpoint("config/port"))
            .json(&json!({
                "service": self.service,
                "port": self.port,
                "skip_health_check": self.skip_health
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        if let Some(error) = result.get("error") {
            println!("{}", format!("✗ {}", error).red());
            Err(anyhow::anyhow!("Error updating port"))
        } else {
            println!(
                "{}",
                format!("✓ {}", result["message"].as_str().unwrap_or("Updated")).green()
            );
            Ok(())
        }
    }
}
