use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) server: ServerConfig,
    pub(crate) auth: AuthConfig,
}

#[derive(Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) workers: u16,
}

#[derive(Deserialize)]
pub(crate) struct AuthConfig {
    pub(crate) auth_timeout: u32,
    pub(crate) discord: Option<StandardAuthConfig>,
}

#[derive(Deserialize)]
pub(crate) struct StandardAuthConfig {
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}
