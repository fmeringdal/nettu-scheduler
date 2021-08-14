use chrono::Utc;
use nettu_scheduler_domain::{IntegrationProvider, User};
use tracing::error;

use crate::{CodeTokenRequest, CodeTokenResponse, NettuContext};
use serde::Deserialize;

// https://developers.google.com/identity/protocols/oauth2/web-server#httprest_3

const TOKEN_REFETCH_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v4/token";
const CODE_TOKEN_EXCHANGE_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const REQUIRED_OAUTH_SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/calendar"];

struct RefreshTokenRequest {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
    // Access token expires in specified in seconds
    expires_in: i64,
}

async fn refresh_access_token(req: RefreshTokenRequest) -> Result<RefreshTokenResponse, ()> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("refresh_token", req.refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ];
    let client = reqwest::Client::new();
    let res = client
        .post(TOKEN_REFETCH_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|_| ())?;

    res.json::<RefreshTokenResponse>().await.map_err(|_| ())
}

pub async fn exchange_code_token(req: CodeTokenRequest) -> Result<CodeTokenResponse, ()> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("redirect_uri", req.redirect_uri.as_str()),
        ("code", req.code.as_str()),
        ("grant_type", "authorization_code"),
    ];
    // TODO: query params instead of body ??
    let client = reqwest::Client::new();
    let res = client
        .post(CODE_TOKEN_EXCHANGE_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("1. Unable to exchange code token: {:?}", e);
        })?;

    let res = res.json::<CodeTokenResponse>().await.map_err(|e| {
        error!("2. Unable to exchange code token: {:?}", e);
    })?;

    let scopes = res.scope.split(' ').collect::<Vec<_>>();
    for required_scope in REQUIRED_OAUTH_SCOPES.iter() {
        if !scopes.contains(required_scope) {
            error!(
                "Missing scope: {:?} got scopes: {:?}",
                required_scope, scopes
            );
            return Err(());
        }
    }

    Ok(res)
}

pub async fn get_access_token(user: &User, ctx: &NettuContext) -> Option<String> {
    // Check if user has connected to google
    let mut integrations = ctx.repos.user_integrations.find(&user.id).await.ok()?;
    let integration = integrations
        .iter_mut()
        .find(|i| matches!(i.provider, IntegrationProvider::Google))?;

    let now = Utc::now().timestamp_millis();
    let one_minute_in_millis = 1000 * 60;
    if now + one_minute_in_millis <= integration.access_token_expires_ts {
        // Current access token is still valid for at least one minutes so return it
        return Some(integration.access_token.clone());
    }
    // Access token has or will expire soon, now renew it

    // The account contains the google client id and secret
    let acc_integrations = match ctx.repos.account_integrations.find(&user.account_id).await {
        Ok(acc_integrations) => acc_integrations,
        Err(_) => return None,
    };
    let google_settings = acc_integrations
        .into_iter()
        .find(|i| matches!(i.provider, IntegrationProvider::Google))?;

    let refresh_token_req = RefreshTokenRequest {
        client_id: google_settings.client_id,
        client_secret: google_settings.client_secret,
        refresh_token: integration.refresh_token.clone(),
    };
    let data = refresh_access_token(refresh_token_req).await;
    match data {
        Ok(tokens) => {
            integration.access_token = tokens.access_token;
            let now = Utc::now().timestamp_millis();
            let expires_in_millis = tokens.expires_in * 1000;
            integration.access_token_expires_ts = now + expires_in_millis;
            let access_token = integration.access_token.clone();

            // Update user with updated google tokens
            if let Err(e) = ctx.repos.user_integrations.save(integration).await {
                error!(
                    "Unable to save updated google credentials for user. Error: {:?}",
                    e
                );
            }

            // Return access_token
            Some(access_token)
        }
        Err(e) => {
            error!("Unable to refresh access token for user. Error: {:?}", e);
            None
        }
    }
}
