use std::sync::Arc;

use anyhow::Result;
use ntex::web;

use crate::config::structs::Config;

use super::handlers;

pub(crate) async fn auth_web(config: Config) -> Result<()> {
    let config_arc = Arc::new(config);
    let config_copy = config_arc.clone();

    let address = (config_copy.server.ip.clone(), config_copy.server.port);

    web::HttpServer::new(move || {
        web::App::new()
            .state(config_arc.clone())
            .service(handlers::version)
    })
    .workers(config_copy.server.workers.into())
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
