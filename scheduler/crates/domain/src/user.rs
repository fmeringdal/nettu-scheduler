use crate::{
    shared::entity::{Entity, ID},
    Meta, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: ID,
    pub account_id: ID,
    pub metadata: Metadata,
}

impl User {
    pub fn new(account_id: ID) -> Self {
        Self {
            account_id,
            ..Default::default()
        }
    }
}

impl Entity<ID> for User {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

impl Meta<ID> for User {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntegration {
    pub user_id: ID,
    pub account_id: ID,
    pub provider: IntegrationProvider,
    pub refresh_token: String,
    pub access_token: String,
    pub access_token_expires_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum IntegrationProvider {
    Google,
    Outlook,
}

impl Default for IntegrationProvider {
    fn default() -> Self {
        IntegrationProvider::Google
    }
}

impl From<IntegrationProvider> for String {
    fn from(e: IntegrationProvider) -> Self {
        match e {
            IntegrationProvider::Google => "google".into(),
            IntegrationProvider::Outlook => "outlook".into(),
        }
    }
}

impl From<String> for IntegrationProvider {
    fn from(e: String) -> IntegrationProvider {
        match &e[..] {
            "google" => IntegrationProvider::Google,
            "outlook" => IntegrationProvider::Outlook,
            _ => unreachable!("Invalid provider"),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UserGoogleIntegrationData {
//     pub refresh_token: String,
//     pub access_token: String,
//     pub access_token_expires_ts: i64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UserOutlookIntegrationData {
//     pub refresh_token: String,
//     pub access_token: String,
//     pub access_token_expires_ts: i64,
// }
