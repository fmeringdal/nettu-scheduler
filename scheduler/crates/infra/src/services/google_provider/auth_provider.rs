use nettu_scheduler_domain::{User, UserGoogleIntegrationData, UserIntegrationProvider};

use crate::NettuContext;
use serde::Deserialize;

use super::GoogleCalendarProvider;

// https://developers.google.com/identity/protocols/oauth2/web-server#httprest_3

const TOKEN_REFETCH_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v4/token";
const CODE_TOKEN_EXHANGE_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

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
    expires_in: usize,
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

// Google api actually returns snake case response
pub struct CodeTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
    expires_in: usize,
    refresh_token: String,
}

pub async fn exchange_code_token(req: CodeTokenRequest) -> Result<CodeTokenResponse, ()> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("redirect_uri", req.redirect_uri.as_str()),
        ("code", req.code.as_str()),
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

    // TODO: validate scopes better
    // This should probably be in config or global variable
    let REQUIRED_SCOPES = vec!["test"];
    let scopes = res.scope.split(" ").collect::<Vec<_>>();
    for required_scope in REQUIRED_SCOPES {
        if !scopes.contains(&required_scope) {
            return Err(());
        }
    }

    Ok(res)
}

// TODO: need to update user to check if refresh token has expires
pub async fn get_access_token(user: &User, ctx: &NettuContext) -> Option<String> {
    // 1. Get if user has connected to google
    let mut integration: Option<&UserGoogleIntegrationData> = None;
    for user_integration in &user.integrations {
        match user_integration {
            UserIntegrationProvider::Google(data) => {
                integration = Some(data);
            }
        }
    }
    if integration.is_none() {
        return None;
    }
    let integration = integration.unwrap();

    // 2. Fetch credentials from mongodb given user id and refersh_token
    // TODO: create repo

    // 3. If access token is still valid for 5 minutes then return it

    // 4. Refresh access token
    let refresh_token_req = RefreshTokenRequest {
        client_id: "TODO".to_string(),
        client_secret: "TODO".to_string(),
        refresh_token: integration.refresh_token.clone(),
    };
    let data = refresh_access_token(refresh_token_req).await;

    // 5. update mongodb with new token data

    // 6. return access_token

    Some("TODO".into())
}
