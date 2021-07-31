use crate::{
    shared::entity::{Entity, ID},
    Meta, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct User {
    pub id: ID,
    pub account_id: ID,
    pub metadata: Metadata,
}

impl User {
    pub fn new(account_id: ID) -> Self {
        Self {
            id: Default::default(),
            account_id,
            metadata: Default::default(),
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
    pub provider: UserIntegrationProvider,
    pub refresh_token: String,
    pub access_token: String,
    pub access_token_expires_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserIntegrationProvider {
    Google,
    Outlook,
}

impl Into<String> for UserIntegrationProvider {
    fn into(self) -> String {
        match self {
            Self::Google => "google".into(),
            Self::Outlook => "outlook".into(),
        }
    }
}

impl Into<UserIntegrationProvider> for String {
    fn into(self) -> UserIntegrationProvider {
        match &self[..] {
            "google" => UserIntegrationProvider::Google,
            "outlook" => UserIntegrationProvider::Outlook,
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
