use crate::{shared::mongo_repo::MongoPersistence, user::domain::User};

use super::IUserRepo;
use crate::shared::mongo_repo;
use mongodb::{
    bson::{doc, from_bson, to_bson, Bson, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
struct UserMongo {
    _id: String,
    account_id: String,
    external_id: String,
}

impl UserMongo {
    pub fn to_domain(&self) -> User {
        User {
            id: self._id.clone(),
            account_id: self.account_id.clone(),
            external_id: self.external_id.clone(),
        }
    }

    pub fn from_domain(user: &User) -> Self {
        Self {
            _id: user.id.clone(),
            account_id: user.account_id.clone(),
            external_id: user.external_id.clone(),
        }
    }
}

impl MongoPersistence for User {
    fn to_domain(doc: Document) -> Self {
        let parsed: UserMongo = from_bson(Bson::Document(doc)).unwrap();
        parsed.to_domain()
    }

    fn to_persistence(&self) -> Document {
        let raw = UserMongo::from_domain(self);
        to_bson(&raw).unwrap().as_document().unwrap().to_owned()
    }

    fn get_persistence_id(&self) -> anyhow::Result<crate::shared::mongo_repo::MongoPersistenceID> {
        Ok(mongo_repo::MongoPersistenceID::String(self.id.clone()))
    }
}
