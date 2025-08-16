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
    pub health_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub domain: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
    pub routes: Vec<Route>,
    pub api_port: u16,
    pub proxy_port: u16,
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

    #[allow(dead_code)]
    pub async fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        let yaml = serde_yaml::to_string(&*config)?;
        fs::write("config.yaml", yaml).await?;
        Ok(())
    }

    pub async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.yaml").await?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        log::info!("Config: {:?}", config);
        Ok(config)
    }
}
