use crate::user::domain::User;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

use super::IUserRepo;

pub struct UserRepo {
    collection: RwLock<Collection>,
}

// RwLock is Send + Sync
unsafe impl Send for UserRepo {}
unsafe impl Sync for UserRepo {}

impl UserRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("users")),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for UserRepo {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let _res = coll.insert_one(to_persistence(user), None).await;
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let filter = doc! {
            "_id": ObjectId::with_string(&user.id)?
        };
        let _res = coll.update_one(filter, to_persistence(user), None).await;
        Ok(())
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        let filter = doc! {
            "_id": user_id,
        };
        let coll = self.collection.read().await;
        let res = coll.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let user = to_domain(doc.unwrap());
                Some(user)
            }
            _ => None,
        }
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        let filter = doc! {
            "_id": ObjectId::with_string(user_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one_and_delete(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let user = to_domain(doc.unwrap());
                Some(user)
            }
            _ => None,
        }
    }
}

fn to_persistence(user: &User) -> Document {
    let raw = doc! {
        "id": to_bson(&user.id).unwrap(),
        "account_id": to_bson(&user.account_id).unwrap(),
        "external_id": to_bson(&user.external_id).unwrap(),
    };

    raw
}

fn to_domain(raw: Document) -> User {
    let user = User {
        id: from_bson(raw.get("_id").unwrap().clone()).unwrap(),
        account_id: from_bson(raw.get("account_id").unwrap().clone()).unwrap(),
        external_id: from_bson(raw.get("external_id").unwrap().clone()).unwrap(),
    };

    user
}
