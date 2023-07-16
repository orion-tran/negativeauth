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

    let reqwest = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            " (https://github.com/orion-tran/negativeauth, ",
            env!("CARGO_PKG_VERSION"),
            ")",
        ))
        .build()?;

    web::HttpServer::new(move || {
        web::App::new()
            .state(config.to_owned())
            .state(clients_arc.to_owned())
            .state(redis_connection.to_owned())
            .state(reqwest.to_owned())
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
