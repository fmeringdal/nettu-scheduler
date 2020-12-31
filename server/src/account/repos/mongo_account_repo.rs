use crate::account::domain::Account;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

use super::IAccountRepo;

pub struct AccountRepo {
    collection: RwLock<Collection>,
}

// RwLock is Send + Sync
unsafe impl Send for AccountRepo {}
unsafe impl Sync for AccountRepo {}

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
        let coll = self.collection.read().await;
        let _res = coll.insert_one(to_persistence(account), None).await;
        Ok(())
    }

    async fn save(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let filter = doc! {
            "_id": ObjectId::with_string(&account.id)?
        };
        let _res = coll.update_one(filter, to_persistence(account), None).await;
        Ok(())
    }

    async fn find(&self, account_id: &str) -> Option<Account> {
        let filter = doc! {
            "_id": ObjectId::with_string(account_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let account = to_domain(doc.unwrap());
                Some(account)
            }
            _ => None,
        }
    }

    async fn delete(&self, account_id: &str) -> Option<Account> {
        let filter = doc! {
            "_id": ObjectId::with_string(account_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one_and_delete(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let account = to_domain(doc.unwrap());
                Some(account)
            }
            _ => None,
        }
    }
}

fn to_persistence(account: &Account) -> Document {
    let raw = doc! {
        "_id": ObjectId::with_string(&account.id).unwrap()
    };

    raw
}

fn to_domain(raw: Document) -> Account {
    let id = match raw.get("_id").unwrap() {
        Bson::ObjectId(oid) => oid.to_string(),
        _ => unreachable!("This should not happen"),
    };

    let account = Account {
        id,
        public_key_b64: from_bson(raw.get("public_key_b64").unwrap().clone()).unwrap(),
    };

    account
}
