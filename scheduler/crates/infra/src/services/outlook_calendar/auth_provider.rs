use chrono::Utc;
use nettu_scheduler_domain::{User, UserIntegrationProvider, UserOutlookIntegrationData};
use tracing::log::warn;

use crate::NettuContext;
use serde::Deserialize;

// https://developers.google.com/identity/protocols/oauth2/web-server#httprest_3

const TOKEN_REFETCH_ENDPOINT: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const CODE_TOKEN_EXHANGE_ENDPOINT: &str =
    "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const REQUIRED_OAUTH_SCOPES: [&str; 2] = [
    "https://graph.microsoft.com/calendars.readwrite",
    "offline_access",
];

// https://docs.microsoft.com/en-us/graph/auth-v2-user#request
struct RefreshTokenRequest {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    refresh_token: String,
    scope: String,
}

// https://docs.microsoft.com/en-us/graph/auth-v2-user#response
#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    refresh_token: String,
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
        ("redirect_uri", req.redirect_uri.as_str()),
        ("refresh_token", req.refresh_token.as_str()),
        ("scope", req.scope.as_str()),
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

// https://docs.microsoft.com/en-us/graph/auth-v2-user#token-request
pub struct CodeTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub code: String,
}

// https://docs.microsoft.com/en-us/graph/auth-v2-user#token-response
#[derive(Debug, Deserialize)]
pub struct CodeTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

pub async fn exchange_code_token(req: CodeTokenRequest) -> Result<CodeTokenResponse, ()> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("redirect_uri", req.redirect_uri.as_str()),
        ("code", req.code.as_str()),
        ("scope", &REQUIRED_OAUTH_SCOPES.join(" ")),
        ("grant_type", "authorization_code"),
    ];

    let client = reqwest::Client::new();
    let res = client
        .post(CODE_TOKEN_EXHANGE_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|_| ())?;

    let res = res.json::<CodeTokenResponse>().await.map_err(|_| ())?;

    let scopes = res.scope.split(" ").collect::<Vec<_>>();
    for required_scope in REQUIRED_OAUTH_SCOPES.iter() {
        if !scopes.contains(&required_scope) {
            return Err(());
        }
    }

    Ok(res)
}

pub async fn get_access_token(user: &mut User, ctx: &NettuContext) -> Option<String> {
    // Check if user has connected to outlook
    let mut integration: Option<&mut UserOutlookIntegrationData> = None;
    for user_integration in &mut user.integrations {
        match user_integration {
            UserIntegrationProvider::Outlook(data) => {
                integration = Some(data);
            }
            _ => (),
        }
    }
    if integration.is_none() {
        return None;
    }
    let integration = integration.unwrap();

    let now = Utc::now().timestamp_millis();
    let one_minute_in_millis = 1000 * 60;
    if now + one_minute_in_millis <= integration.access_token_expires_ts {
        // Current acces token is still valid for at least one minutes so return it
        return Some(integration.access_token.clone());
    }
    // Access token has or will expire soon, now renew it

    // The account contains the google client id and secret
    let account = match ctx.repos.accounts.find(&user.account_id).await {
        Some(a) => a,
        None => return None,
    };
    let outlook_settings = match account.settings.outlook {
        Some(settings) => settings,
        None => return None,
    };

    let refresh_token_req = RefreshTokenRequest {
        client_id: outlook_settings.client_id,
        client_secret: outlook_settings.client_secret,
        refresh_token: integration.refresh_token.clone(),
        redirect_uri: outlook_settings.redirect_uri.clone(),
        scope: REQUIRED_OAUTH_SCOPES.join(" "),
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
            if let Err(e) = ctx.repos.users.save(&user).await {
                warn!(
                    "Unable to save updated google credentials for user. Error: {:?}",
                    e
                );
            }

            // Return access_token
            Some(access_token)
        }
        Err(e) => {
            warn!("Unable to refresh access token for user. Error: {:?}", e);
            None
        }
    }
}
