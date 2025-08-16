use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Env {
    pub port: u16,
    pub host: Cow<'static, str>,
    pub proxy_port: u16,
}

impl Env {
    pub fn new() -> Self {
        let port = match std::env::var("PORT") {
            Ok(port) => port.parse().unwrap_or(1143),
            Err(_) => 1143,
        };
        let host = match std::env::var("HOST") {
            Ok(host) => Cow::Owned(host),
            Err(_) => Cow::Owned("http://localhost/".to_string()),
        };
        let proxy_port = match std::env::var("PROXY_PORT") {
            Ok(proxy_port) => proxy_port.parse().unwrap_or(1144),
            Err(_) => 1144,
        };

        Self {
            port,
            host,
            proxy_port,
        }
    }
}
