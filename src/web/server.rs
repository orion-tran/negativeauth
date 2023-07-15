use std::sync::Arc;

use anyhow::Result;
use ntex::web;

use crate::{auth::authclients::AuthClients, config::structs::Config};

use super::handlers;

pub(crate) async fn auth_web(
    config: Config,
    clients: AuthClients,
    redis_connection: rustis::client::Client,
) -> Result<()> {
    let config_copy = config.clone();
    let clients_arc = Arc::new(clients);

    let address = (config_copy.server.ip.clone(), config_copy.server.port);

    web::HttpServer::new(move || {
        web::App::new()
            .state(config.clone())
            .state(clients_arc.clone())
            .state(redis_connection.clone())
            .service(handlers::version)
            .service(handlers::authorize_discord)
            .service(handlers::verify_discord)
    })
    .workers(config_copy.server.workers.into())
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
