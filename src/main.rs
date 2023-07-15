use auth::authclients::init_clients;
use config::{loader::read_config, structs::Config};
use tracing::{error, info};
use web::server::auth_web;

mod auth;
mod config;
mod web;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            error!("error in reading config: {}", e);
            return Ok(());
        }
    };

    let auth_clients = match init_clients(&config) {
        Ok(clients) => clients,
        Err(e) => {
            error!("could not create auth clients: {}", e);
            return Ok(());
        }
    };

    let redis_connection = match connect_redis(&config).await {
        Ok(client) => client,
        Err(e) => {
            error!("error in connecting to redis: {}", e);
            return Ok(());
        }
    };

    if let Err(e) = auth_web(config, auth_clients, redis_connection).await {
        error!("error in web server: {}", e);
    }

    Ok(())
}

async fn connect_redis(config: &Config) -> anyhow::Result<rustis::client::Client> {
    Ok(rustis::client::Client::connect(config.redis.uri.to_string()).await?)
}
