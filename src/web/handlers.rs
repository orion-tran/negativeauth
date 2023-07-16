use std::{io::Read, sync::Arc};

use cookie::Cookie;
use ntex::web::{
    self,
    types::{Query, State},
    HttpRequest,
};
use oauth2::{
    basic::BasicClient, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse,
};
use rustis::commands::{GenericCommands, StringCommands};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{auth::authclients::AuthClients, config::structs::Config};

#[derive(Serialize)]
struct Version {
    version: String,
}

#[derive(Deserialize)]
pub(crate) struct AuthOptions {
    redirect: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AuthSession {
    verifier: PkceCodeVerifier,
    redirect_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AuthState {
    code: String,
    state: String,
}

#[web::get("/version")]
pub(crate) async fn version(request: HttpRequest) -> impl web::Responder {
    debug!("cookies for version: {:?}", request.headers().get("cookie"));
    web::HttpResponse::Ok().json(&Version {
        version: env!("CARGO_PKG_VERSION_MAJOR").to_string(),
    })
}

async fn authorize(
    request: HttpRequest,
    auth_client: &BasicClient,
    scopes: &'static [&'static str],
    key: &'static str,
    auth_options: Query<AuthOptions>,
    config: State<Config>,
    redis_connection: State<rustis::client::Client>,
) -> ntex::http::Response {
    if let Some(Some(cookie)) = request
        .headers()
        .get("cookie")
        .map(|cookie| cookie.to_str().ok())
    {
        let cookie_name = format!("na_{}_pending", key);
        if let Some(matched_cookie) = Cookie::split_parse(cookie)
            .filter_map(|it| it.ok())
            .find(|it| it.name() == cookie_name)
        {
            if matched_cookie.value().is_ascii() && matched_cookie.value() != "none" {
                debug!("deleting existing cookie: {}", matched_cookie.value());
                if let Err(e) = redis_connection
                    .del(format!("request_{}_{}", key, matched_cookie.value()))
                    .await
                {
                    error!("could not delete old pending cookie in redis: {}", e)
                }
            }
        }
    }

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_request = auth_client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);

    for scope in scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }

    let (auth_url, csrf_token) = auth_request.url();

    let auth_session = match serde_json::to_string(&AuthSession {
        verifier: pkce_verifier,
        redirect_url: auth_options.redirect.clone(),
    }) {
        Ok(v) => v,
        Err(e) => {
            error!("Could not serialize auth session value: {}", e);
            return web::HttpResponse::InternalServerError().finish();
        }
    };

    match redis_connection
        .set_with_options(
            format!("request_{}_{}", key, csrf_token.secret()),
            auth_session,
            rustis::commands::SetCondition::NX,
            rustis::commands::SetExpiration::Ex(config.auth.auth_timeout.into()),
            false,
        )
        .await
    {
        Ok(v) => {
            if !v {
                warn!("Pre-existing CSRF token generated, this should NEVER happen!");
                return web::HttpResponse::InternalServerError().finish();
            }
        }
        Err(e) => {
            error!("Error in setting redis key: {}", e);
            return web::HttpResponse::InternalServerError().finish();
        }
    }

    web::HttpResponse::SeeOther()
        .header("Location", auth_url.to_string())
        .header(
            "Set-Cookie",
            format!(
                "na_{}_pending={}; Path=/; SameSite=strict; Secure; HttpOnly; Max-Age={}",
                key,
                csrf_token.secret(),
                config.auth.auth_timeout
            ),
        )
        .finish()
}

const DISCORD_SCOPES: &[&str] = &["identify", "email"];

#[web::get("/authorize/discord")]
pub(crate) async fn authorize_discord(
    request: HttpRequest,
    auth_options: Query<AuthOptions>,
    auth_clients: State<Arc<AuthClients>>,
    config: State<Config>,
    redis_connection: State<rustis::client::Client>,
) -> impl web::Responder {
    let discord_client = match &auth_clients.discord_client {
        Some(v) => v,
        None => return web::HttpResponse::MethodNotAllowed().finish(),
    };

    authorize(
        request,
        discord_client,
        DISCORD_SCOPES,
        "discord",
        auth_options,
        config,
        redis_connection,
    )
    .await
}

#[web::get("/verify/discord")]
pub(crate) async fn verify_discord(
    auth_state: Query<AuthState>,
    auth_clients: State<Arc<AuthClients>>,
    config: State<Config>,
    reqwest: State<reqwest::Client>,
    redis_connection: State<rustis::client::Client>,
) -> impl web::Responder {
    let discord_client = match &auth_clients.discord_client {
        Some(v) => v,
        None => return web::HttpResponse::MethodNotAllowed().finish(),
    };

    let request: String = match redis_connection
        .getdel(format!("request_discord_{}", auth_state.state))
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("could not redis get: {}", e);
            return web::HttpResponse::InternalServerError().finish();
        }
    };

    if request.is_empty() {
        return web::HttpResponse::BadRequest().body("Request invalid or expired");
    }

    let session = match serde_json::from_str::<AuthSession>(&request) {
        Ok(v) => v,
        Err(e) => {
            error!("could not parse invalid redis request: {}", e);
            return web::HttpResponse::InternalServerError().finish();
        }
    };

    let access_token = match discord_client
        .exchange_code(AuthorizationCode::new(auth_state.code.clone()))
        .set_pkce_verifier(session.verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            debug!(
                "authentication flow failed for CSRF token {} because: {}",
                auth_state.code, e
            );
            return web::HttpResponse::Unauthorized().body("Authentication failed!");
        }
    };

    match access_token.scopes() {
        Some(scopes) => {
            for required_scope in DISCORD_SCOPES {
                if scopes
                    .iter()
                    .any(|it: &Scope| it.as_str() == *required_scope)
                {
                    continue;
                }

                if let Err(e) = discord_client.revoke_token(access_token.access_token().into()) {
                    error!("couldn't revoke access token (with missing scopes): {}", e);
                }

                return web::HttpResponse::Unauthorized().body("Missing authentication scopes!");
            }
        }
        None => {
            if let Err(e) = discord_client.revoke_token(access_token.access_token().into()) {
                error!("couldn't revoke access token (with missing scopes): {}", e);
            }

            return web::HttpResponse::Unauthorized().body("Authentication scopes required!");
        }
    }

    let a = reqwest
        .get("https://discord.com/api/v10/users/@me")
        .bearer_auth(access_token.access_token().secret())
        .send()
        .await;

    if let Ok(res) = a {
        info!("{}", res.text().await.expect("ata"));
    }

    web::HttpResponse::Ok()
        .header(
            "Set-Cookie",
            "na_discord_pending=none; Path=/; SameSite=strict; Secure; HttpOnly; Max-Age=0",
        )
        .body("Authentication successful!")
}
