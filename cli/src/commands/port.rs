use colored::Colorize;
use reqwest::Client;
use serde_json::json;

use crate::Cli;
use anyhow::Result;

pub async fn execute(cli: Cli, client: Client, service: &str, port: u16) -> Result<()> {
    println!(
        "{}",
        format!("Updating {} to port {}...", service, port).blue()
    );

    let response = client
        .post(format!("{}/config/port", cli.api_url))
        .json(&json!({
            "service": service,
            "port": port
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
