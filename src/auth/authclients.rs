use anyhow::Result;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl,
};

use crate::config::structs::Config;

pub(crate) struct AuthClients {
    pub(crate) discord_client: Option<BasicClient>,
}

pub(crate) fn init_clients(config: &Config) -> Result<AuthClients> {
    let discord_client = match &config.auth.discord {
        Some(v) => Some(
            BasicClient::new(
                ClientId::new(v.client_id.clone()),
                Some(ClientSecret::new(v.client_secret.clone())),
                AuthUrl::new("https://discord.com/oauth2/authorize".to_string())?,
                Some(TokenUrl::new(
                    "https://discord.com/api/oauth2/token".to_string(),
                )?),
            )
            .set_revocation_uri(RevocationUrl::new(
                "https://discord.com/api/oauth2/token/revoke".to_string(),
            )?)
            .set_redirect_uri(RedirectUrl::new(format!(
                "{}{}",
                config.web.base_url, "verify/discord"
            ))?),
        ),
        _ => None,
    };

    Ok(AuthClients { discord_client })
}
