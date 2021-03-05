use crate::shared::entity::{Entity, ID};
use nettu_scheduler_utils::create_random_secret;

const API_KEY_LEN: usize = 30;

/// An `Account` acts as a kind of namespace and lets multiple different
/// applications use the same instance of this server without interfering
/// with each other.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: ID,
    pub public_jwt_key: Option<String>,
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
                        key: Account::generate_secret_api_key(),
                    });
                }
            }
            None => {
                self.webhook = None;
            }
        };
        true
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
            id: Default::default(),
            public_jwt_key: None,
            secret_api_key: Self::generate_secret_api_key(),
            settings: Default::default(),
        }
    }

    pub fn generate_secret_api_key() -> String {
        let rand_secret = create_random_secret(API_KEY_LEN);
        format!("sk_{}", rand_secret)
    }

    pub fn set_public_jwt_key(&mut self, key: Option<String>) -> anyhow::Result<()> {
        match key {
            Some(key) => {
                jsonwebtoken::DecodingKey::from_rsa_pem(key.as_bytes().as_ref())?;
                self.public_jwt_key = Some(key);
                Ok(())
            }
            None => {
                self.public_jwt_key = None;
                Ok(())
            }
        }
    }
}

impl Entity for Account {
    fn id(&self) -> &ID {
        &self.id
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
        assert!(acc.secret_api_key.starts_with("sk_"));
        assert!(acc.secret_api_key.len() > API_KEY_LEN);
    }

    #[test]
    fn it_rejects_invalid_public_key() {
        let mut acc = Account::new();
        assert!(acc.set_public_jwt_key(Some("badpem".into())).is_err());
        assert!(acc.public_jwt_key.is_none());
    }

    #[test]
    fn it_accepts_valid_public_key() {
        let mut acc = Account::new();

        let pub_key = std::fs::read("../api/config/test_public_rsa_key.crt").unwrap();
        let pub_key = String::from_utf8(pub_key).expect("Test public key to be valid utf8");

        assert!(acc.set_public_jwt_key(Some(pub_key.clone())).is_ok());
        assert_eq!(acc.public_jwt_key.clone().unwrap(), pub_key);

        // Remove pub key
        assert!(acc.set_public_jwt_key(None).is_ok());
        assert!(acc.public_jwt_key.is_none());
    }
}
