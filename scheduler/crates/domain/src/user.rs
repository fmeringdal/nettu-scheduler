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
    pub integrations: Vec<UserIntegrationProvider>,
}

impl User {
    pub fn new(account_id: ID) -> Self {
        Self {
            id: Default::default(),
            account_id,
            metadata: Default::default(),
            integrations: Default::default(),
        }
    }
}

impl Entity for User {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Meta for User {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum UserIntegrationProvider {
    Google(UserGoogleIntegrationData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGoogleIntegrationData {
    pub refresh_token: String,
    pub access_token: String,
    pub access_token_expires_ts: usize,
}
