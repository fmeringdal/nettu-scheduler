pub mod google_calendar;
pub mod outlook_calendar;

use nettu_scheduler_domain::IntegrationProvider;
use serde::Deserialize;

#[derive(Debug)]
pub struct FreeBusyProviderQuery {
    pub calendar_ids: Vec<String>,
    pub start: i64,
    pub end: i64,
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

#[async_trait::async_trait]
pub trait ProviderOAuth {
    async fn exchange_code_token(&self, req: CodeTokenRequest) -> Result<CodeTokenResponse, ()>;
}

#[async_trait::async_trait]
impl ProviderOAuth for IntegrationProvider {
    async fn exchange_code_token(&self, req: CodeTokenRequest) -> Result<CodeTokenResponse, ()> {
        match *self {
            Self::Google => google_calendar::auth_provider::exchange_code_token(req).await,
            Self::Outlook => outlook_calendar::auth_provider::exchange_code_token(req).await,
        }
    }
}
