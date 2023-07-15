use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct Config {
    pub(crate) server: ServerConfig,
    pub(crate) web: WebConfig,
    pub(crate) redis: RedisConfig,
    pub(crate) auth: AuthConfig,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ServerConfig {
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) workers: u16,
}

#[derive(Deserialize, Clone)]
pub(crate) struct AuthConfig {
    pub(crate) auth_timeout: u32,
    pub(crate) discord: Option<StandardAuthConfig>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct StandardAuthConfig {
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}

#[derive(Deserialize, Clone)]
pub(crate) struct RedisConfig {
    pub(crate) uri: String,
}

#[derive(Deserialize, Clone)]
pub(crate) struct WebConfig {
    pub(crate) base_url: String,
}
