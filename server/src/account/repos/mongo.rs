use crate::account::domain::{Account, AccountSettings};

use super::IAccountRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct AccountRepo {
    collection: Collection,
}

impl AccountRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("accounts"),
        }
    }
}

#[async_trait::async_trait]
impl IAccountRepo for AccountRepo {
    async fn insert(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert::<_, AccountMongo>(&self.collection, account).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<_, AccountMongo>(&self.collection, account).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, account_id: &str) -> Option<Account> {
        let id = match ObjectId::with_string(account_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find::<_, AccountMongo>(&self.collection, &id).await
    }

    async fn find_many(&self, accounts_ids: &[String]) -> Result<Vec<Account>, Box<dyn Error>> {
        let filter = doc! {
            "event_id": {
                "$in": accounts_ids
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

    async fn delete(&self, account_id: &str) -> Option<Account> {
        let id = match ObjectId::with_string(account_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete::<_, AccountMongo>(&self.collection, &id).await
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
    pub webhook_url: Option<String>,
    pub webhook_key: Option<String>,
}

impl<'de> MongoDocument<Account> for AccountMongo {
    fn to_domain(&self) -> Account {
        Account {
            id: self._id.to_string(),
            public_key_b64: self.public_key_b64.clone(),
            secret_api_key: self.secret_api_key.clone(),
            settings: AccountSettings {
                webhook_key: self.settings.webhook_key.clone(),
                webhook_url: self.settings.webhook_url.clone(),
            },
        }
    }

    fn from_domain(account: &Account) -> Self {
        Self {
            _id: ObjectId::with_string(&account.id).unwrap(),
            public_key_b64: account.public_key_b64.clone(),
            secret_api_key: account.secret_api_key.clone(),
            settings: AccountSettingsMongo {
                webhook_key: account.settings.webhook_key.clone(),
                webhook_url: account.settings.webhook_url.clone(),
            },
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
