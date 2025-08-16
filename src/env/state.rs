use super::app::Env;
use dotenv::dotenv;

#[derive(Clone)]
pub struct AppState {
    pub port: u16,
    pub proxy_port: u16,
}

impl AppState {
    pub fn from_env() -> Self {
        dotenv().ok();

        let env = Env::new();

        Self {
            port: env.port,
            proxy_port: env.proxy_port,
        }
    }
}
