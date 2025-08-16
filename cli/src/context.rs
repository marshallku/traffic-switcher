use reqwest::Client;

#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub api_url: String,
}

impl Context {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }

    pub fn api_endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.api_url, path.trim_start_matches('/'))
    }
}