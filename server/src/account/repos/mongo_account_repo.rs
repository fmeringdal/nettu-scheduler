use crate::account::domain::Account;

use super::IAccountRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

pub struct AccountRepo {
    collection: RwLock<Collection>,
}

impl AccountRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("accounts")),
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

impl MongoPersistence for Account {
    fn to_domain(doc: Document) -> Self {
        let id = match doc.get("_id").unwrap() {
            Bson::ObjectId(oid) => oid.to_string(),
            _ => unreachable!("This should not happen"),
        };

        let public_key_b64 = match doc.get("public_key_b64") {
            Some(bson) => from_bson(bson.clone()).unwrap_or(None),
            None => None,
        };

        let account = Account {
            id,
            public_key_b64,
            secret_api_key: from_bson(doc.get("secret_api_key").unwrap().clone()).unwrap(),
        };

        account
    }

    fn to_persistence(&self) -> Document {
        let mut raw = doc! {
            "_id": ObjectId::with_string(&self.id).unwrap(),
            "secret_api_key": to_bson(&self.secret_api_key).unwrap()
        };
        if let Ok(public_key_b64) = to_bson(&self.public_key_b64) {
            raw.insert("public_key_b64", public_key_b64);
        }

        raw
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
