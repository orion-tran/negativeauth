use std::sync::Arc;

use ntex::web::{
    self,
    types::{Query, State},
};
use oauth2::{CsrfToken, HttpResponse, PkceCodeChallenge, Scope};
use serde::{Deserialize, Serialize};

use crate::auth::authclients::AuthClients;

#[derive(Serialize)]
struct Version {
    version: String,
}

#[derive(Deserialize)]
pub(crate) struct AuthOptions {
    redirect: Option<String>,
}

#[web::get("/version")]
pub(crate) async fn version() -> impl web::Responder {
    web::HttpResponse::Ok().json(&Version {
        version: env!("CARGO_PKG_VERSION_MAJOR").to_string(),
    })
}

#[web::get("/authorize/discord")]
pub(crate) async fn authorize_discord(
    auth_clients: State<Arc<AuthClients>>,
    auth_options: Query<AuthOptions>,
) -> impl web::Responder {
    let discord_client = match &auth_clients.discord_client {
        Some(v) => v,
        None => return web::HttpResponse::MethodNotAllowed().finish(),
    };

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token) = discord_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    web::HttpResponse::SeeOther()
        .header("Location", auth_url.to_string())
        .finish()
}
