use crate::{shared::mongo_repo::MongoPersistence, user::domain::User};

use super::IUserRepo;
use crate::shared::mongo_repo;
use mongodb::{
    bson::{doc, from_bson, to_bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

pub struct UserRepo {
    collection: RwLock<Collection>,
}

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
        match mongo_repo::insert(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::find(&self.collection, &id).await
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::delete(&self.collection, &id).await
    }
}

impl MongoPersistence for User {
    fn to_domain(doc: Document) -> Self {
        let user = User {
            id: from_bson(doc.get("_id").unwrap().clone()).unwrap(),
            account_id: from_bson(doc.get("account_id").unwrap().clone()).unwrap(),
            external_id: from_bson(doc.get("external_id").unwrap().clone()).unwrap(),
        };

        user
    }

    fn to_persistence(&self) -> Document {
        let raw = doc! {
            "id": to_bson(&self.id).unwrap(),
            "account_id": to_bson(&self.account_id).unwrap(),
            "external_id": to_bson(&self.external_id).unwrap(),
        };

        raw
    }

    fn get_persistence_id(&self) -> anyhow::Result<crate::shared::mongo_repo::MongoPersistenceID> {
        Ok(mongo_repo::MongoPersistenceID::String(self.id.clone()))
    }
}
