use nettu_scheduler_domain::{Account, AccountSettings, AccountWebhookSettings, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDTO {
    pub id: ID,
    pub public_jwt_key: Option<String>,
    pub settings: AccountSettingsDTO,
}

impl AccountDTO {
    pub fn new(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            public_jwt_key: account.public_jwt_key.clone(),
            settings: AccountSettingsDTO::new(&account.settings),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSettingsDTO {
    pub webhook: Option<AccountWebhookSettingsDTO>,
}

impl AccountSettingsDTO {
    pub fn new(settings: &AccountSettings) -> Self {
        let webhook_settings = settings
            .webhook
            .as_ref()
            .map(|webhook| AccountWebhookSettingsDTO::new(webhook));

        Self {
            webhook: webhook_settings,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWebhookSettingsDTO {
    pub url: String,
    pub key: String,
}

impl AccountWebhookSettingsDTO {
    pub fn new(settings: &AccountWebhookSettings) -> Self {
        Self {
            url: settings.url.clone(),
            key: settings.key.clone(),
        }
    }
}
