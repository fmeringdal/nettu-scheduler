use crate::account::domain::Account;

use super::IAccountRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::RwLock;

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
        match mongo_repo::insert(&self.collection, account).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, account).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, account_id: &str) -> Option<Account> {
        let id = match ObjectId::with_string(account_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find(&self.collection, &id).await
    }

    async fn find_by_apikey(&self, api_key: &str) -> Option<Account> {
        let filter = doc! {
            "secret_api_key": api_key
        };
        mongo_repo::find_one_by(&self.collection, filter).await
    }

    async fn delete(&self, account_id: &str) -> Option<Account> {
        let id = match ObjectId::with_string(account_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete(&self.collection, &id).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountMongo {
    pub _id: ObjectId,
    pub public_key_b64: Option<String>,
    pub secret_api_key: String,
}

impl AccountMongo {
    pub fn to_domain(&self) -> Account {
        Account {
            id: self._id.to_string(),
            public_key_b64: self.public_key_b64.clone(),
            secret_api_key: self.secret_api_key.clone(),
        }
    }

    pub fn from_domain(account: &Account) -> Self {
        Self {
            _id: ObjectId::with_string(&account.id).unwrap(),
            public_key_b64: account.public_key_b64.clone(),
            secret_api_key: account.secret_api_key.clone(),
        }
    }
}

impl MongoPersistence for Account {
    fn to_domain(doc: Document) -> Self {
        let doc: AccountMongo = from_bson(Bson::Document(doc)).unwrap();
        doc.to_domain()
    }

    fn to_persistence(&self) -> Document {
        let doc = AccountMongo::from_domain(self);
        to_bson(&doc).unwrap().as_document().unwrap().to_owned()
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
