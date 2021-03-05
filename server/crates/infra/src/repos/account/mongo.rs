use super::IAccountRepo;
use crate::repos::shared::mongo_repo::{self};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Account, AccountSettings, AccountWebhookSettings, ID};
use serde::{Deserialize, Serialize};

pub struct MongoAccountRepo {
    collection: Collection,
}

impl MongoAccountRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("accounts"),
        }
    }
}

#[async_trait::async_trait]
impl IAccountRepo for MongoAccountRepo {
    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        mongo_repo::insert::<_, AccountMongo>(&self.collection, account).await
    }

    async fn save(&self, account: &Account) -> anyhow::Result<()> {
        mongo_repo::save::<_, AccountMongo>(&self.collection, account).await
    }

    async fn find(&self, account_id: &ID) -> Option<Account> {
        let oid = account_id.inner_ref();
        mongo_repo::find::<_, AccountMongo>(&self.collection, &oid).await
    }

    async fn find_many(&self, accounts_ids: &[ID]) -> anyhow::Result<Vec<Account>> {
        let filter = doc! {
            "event_id": {
                "$in": accounts_ids.iter().map(|id| id.inner_ref()).collect::<Vec<_>>()
            }
        };

        mongo_repo::find_many_by::<_, AccountMongo>(&self.collection, filter).await
    }

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let filter = doc! {
            "secret_api_key": api_key
        };
        mongo_repo::find_one_by::<_, AccountMongo>(&self.collection, filter).await
    }

    async fn find_by_webhook_url(&self, url: &str) -> Option<Account> {
        let filter = doc! {
            "settings.webhook.url": url
        };
        mongo_repo::find_one_by::<_, AccountMongo>(&self.collection, filter).await
    }

    async fn delete(&self, account_id: &ID) -> Option<Account> {
        let oid = account_id.inner_ref();
        mongo_repo::delete::<_, AccountMongo>(&self.collection, &oid).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountMongo {
    pub _id: ObjectId,
    pub public_key_b64: Option<String>,
    pub secret_api_key: String,
    pub settings: AccountSettingsMongo,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountSettingsMongo {
    pub webhook: Option<AccountWebhookSettingsMongo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountWebhookSettingsMongo {
    pub url: String,
    pub key: String,
}

impl<'de> MongoDocument<Account> for AccountMongo {
    fn to_domain(&self) -> Account {
        let mut settings = AccountSettings { webhook: None };
        if let Some(webhook_settings) = self.settings.webhook.as_ref() {
            settings.webhook = Some(AccountWebhookSettings {
                url: webhook_settings.url.to_owned(),
                key: webhook_settings.key.to_owned(),
            });
        }

        Account {
            id: ID::from(self._id.clone()),
            public_jwt_key: self.public_key_b64.clone(),
            secret_api_key: self.secret_api_key.clone(),
            settings,
        }
    }

    fn from_domain(account: &Account) -> Self {
        let mut settings = AccountSettingsMongo { webhook: None };
        if let Some(webhook_settings) = account.settings.webhook.as_ref() {
            settings.webhook = Some(AccountWebhookSettingsMongo {
                url: webhook_settings.url.to_owned(),
                key: webhook_settings.key.to_owned(),
            });
        }

        Self {
            _id: account.id.inner_ref().clone(),
            public_key_b64: account.public_jwt_key.clone(),
            secret_api_key: account.secret_api_key.clone(),
            settings,
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
