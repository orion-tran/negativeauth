use config::loader::read_config;
use tracing::error;
use web::server::auth_web;

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

    if let Err(e) = auth_web(config).await {
        error!("error in web server {}", e);
    }

    Ok(())
}
