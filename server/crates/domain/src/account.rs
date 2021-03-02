use crate::shared::entity::Entity;
use mongodb::bson::oid::ObjectId;
use nettu_scheduler_utils::create_random_secret;

const API_KEY_LEN: usize = 30;

/// An `Account` acts as a kind of namespace and lets multiple different
/// applications use the same instance of this server without interfering
/// with each other.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub public_key_b64: Option<String>,
    pub secret_api_key: String,
    pub settings: AccountSettings,
}

#[derive(Debug, Clone)]
pub struct AccountSettings {
    pub webhook: Option<AccountWebhookSettings>,
}

#[derive(Debug, Clone)]
pub struct AccountWebhookSettings {
    pub url: String,
    pub key: String,
}

impl AccountSettings {
    pub fn set_webhook_url(&mut self, webhook_url: Option<String>) -> bool {
        match webhook_url {
            Some(url) => {
                if let Ok(parsed_url) = url::Url::parse(&url) {
                    if parsed_url.scheme() != "https" {
                        return false;
                    }
                } else {
                    return false;
                }

                if let Some(mut webhook_settings) = self.webhook.as_mut() {
                    webhook_settings.url = url;
                } else {
                    self.webhook = Some(AccountWebhookSettings {
                        url,
                        key: Self::generate_webhook_key(),
                    });
                }
            }
            None => {
                self.webhook = None;
            }
        };
        true
    }

    fn generate_webhook_key() -> String {
        Account::generate_secret_api_key()
    }
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self { webhook: None }
    }
}

impl Account {
    pub fn new() -> Self {
        Self {
            id: ObjectId::new().to_string(),
            public_key_b64: None,
            secret_api_key: Self::generate_secret_api_key(),
            settings: Default::default(),
        }
    }

    pub fn generate_secret_api_key() -> String {
        let rand_secret = create_random_secret(API_KEY_LEN);
        format!("sk_live_{}", rand_secret)
    }

    pub fn set_public_key_b64(&mut self, public_key_b64: Option<String>) -> anyhow::Result<()> {
        match public_key_b64 {
            Some(public_key_b64) => {
                base64::decode(&public_key_b64)?;
                self.public_key_b64 = Some(public_key_b64);
                Ok(())
            }
            None => {
                self.public_key_b64 = None;
                Ok(())
            }
        }
    }
}

impl Entity for Account {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_creates_account() {
        let acc = Account::new();
        assert!(acc.secret_api_key.starts_with("sk_live_"));
        assert!(acc.secret_api_key.len() > API_KEY_LEN);
    }
}
