use std::{collections::HashMap, sync::Arc};

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheckConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub domain: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_seconds: u64,
}

fn default_path() -> String {
    "/".to_string()
}

fn default_retry_count() -> u32 {
    10
}

fn default_retry_delay() -> u64 {
    1
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            retry_count: default_retry_count(),
            retry_delay_seconds: default_retry_delay(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
    pub routes: Vec<Route>,
    pub api_port: u16,
    pub proxy_port: u16,
    #[serde(default)]
    pub health_check: HealthCheckConfig,
}

#[derive(Clone)]
pub struct AppState {
    pub port: u16,
    pub proxy_port: u16,
    pub config: Arc<RwLock<Config>>,
    pub services_map: Arc<RwLock<HashMap<String, Service>>>,
    pub routes_map: Arc<RwLock<HashMap<String, String>>>,
}

impl AppState {
    pub async fn new() -> Self {
        dotenv().ok();

        let config = Self::load_config().await.unwrap();

        Self {
            port: config.api_port,
            proxy_port: config.proxy_port,
            config: Arc::new(RwLock::new(config.clone())),
            services_map: Arc::new(RwLock::new(
                config
                    .services
                    .into_iter()
                    .map(|s| (s.name.clone(), s))
                    .collect(),
            )),
            routes_map: Arc::new(RwLock::new(
                config
                    .routes
                    .into_iter()
                    .map(|r| (r.domain.clone(), r.service.clone()))
                    .collect(),
            )),
        }
    }

    pub async fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        let yaml = serde_yaml::to_string(&*config)?;
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
        fs::write(config_path, yaml).await?;
        Ok(())
    }

    pub async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
        let config_str = fs::read_to_string(&config_path).await?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        log::info!("Loaded config from: {}", config_path);
        log::info!("Config: {:?}", config);
        Ok(config)
    }

    pub async fn reload_config(&self) -> Result<Config, Box<dyn std::error::Error>> {
        let new_config = Self::load_config().await?;
        *self.config.write().await = new_config.clone();
        Ok(new_config)
    }

    pub async fn update_service_port(
        &self,
        service_name: &str,
        new_port: u16,
        skip_health_check: bool,
    ) -> Result<u16, String> {
        let mut config = self.config.write().await;
        let mut services_map = self.services_map.write().await;

        let service = config
            .services
            .iter_mut()
            .find(|s| s.name == service_name)
            .ok_or_else(|| format!("Service '{}' not found", service_name))?;

        let old_port = service.port;
        service.previous_port = Some(old_port);
        service.port = new_port;

        log::info!(
            "Updating service '{}' from port {} to {} (skip_health_check: {})",
            service_name,
            old_port,
            new_port,
            skip_health_check
        );

        if !skip_health_check {
            let client = reqwest::Client::new();

            let path = service.health_check.as_ref().unwrap().path.clone();
            let url = format!("http://{}:{}{}", service.host, service.port, path);

            let retry_count = service.health_check.as_ref().unwrap().retry_count;
            let retry_delay = service.health_check.as_ref().unwrap().retry_delay_seconds;

            for i in 0..retry_count {
                let response = client.get(url.clone()).send().await;

                log::info!("Response: {:?}", response);

                if response.is_ok() {
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;

                if i == retry_count - 1 {
                    return Err(format!("Service '{}' is not healthy", service_name));
                }
            }
        }

        if let Some(map_service) = services_map.get_mut(service_name) {
            map_service.previous_port = Some(old_port);
            map_service.port = new_port;
        }

        Ok(old_port)
    }
}
