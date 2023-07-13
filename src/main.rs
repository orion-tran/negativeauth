use auth::authclients::init_clients;
use config::loader::read_config;
use tracing::error;
use web::server::auth_web;

mod auth;
mod config;
mod web;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            error!("error in reading config {}", e);
            return Ok(());
        }
    };

    let auth_clients = match init_clients(&config) {
        Ok(clients) => clients,
        Err(e) => {
            error!("could not create auth clients {}", e);
            return Ok(());
        }
    };

    if let Err(e) = auth_web(config, auth_clients).await {
        error!("error in web server {}", e);
    }

    Ok(())
}
