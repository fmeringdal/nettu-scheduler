use mongodb::bson::oid::ObjectId;

use crate::shared::entity::Entity;

use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
const API_KEY_LEN: usize = 30;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub public_key_b64: Option<String>,
    pub secret_api_key: String,
    pub settings: AccountSettings,
}

#[derive(Debug, Clone)]
pub struct AccountSettings {
    pub webhook_url: Option<String>,
    pub webhook_key: Option<String>,
}

impl AccountSettings {
    pub fn set_webhook_url(&mut self, webhook_url: Option<String>) -> Option<String> {
        match webhook_url {
            Some(url) => {
                if self.webhook_url.is_none() {
                    self.webhook_key = Some(Self::generate_webhook_key())
                }
                self.webhook_url = Some(url);
            }
            None => {
                self.webhook_key = None;
                self.webhook_url = None;
            }
        };
        self.webhook_key.clone()
    }

    fn generate_webhook_key() -> String {
        Account::generate_secret_api_key()
    }
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            webhook_url: None,
            webhook_key: None,
        }
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
        let mut rng = rand::thread_rng();

        let rand_string: String = (0..API_KEY_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("sk_live_{}", rand_string)
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
