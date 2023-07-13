use std::sync::Arc;

use anyhow::Result;
use ntex::web;

use crate::{auth::authclients::AuthClients, config::structs::Config};

use super::handlers;

pub(crate) async fn auth_web(config: Config, clients: AuthClients) -> Result<()> {
    let config_arc = Arc::new(config);
    let config_copy = config_arc.clone();

    let clients_arc = Arc::new(clients);

    let address = (config_copy.server.ip.clone(), config_copy.server.port);

    web::HttpServer::new(move || {
        web::App::new()
            .state(config_arc.clone())
            .state(clients_arc.clone())
            .service(handlers::version)
            .service(handlers::authorize_discord)
    })
    .workers(config_copy.server.workers.into())
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
