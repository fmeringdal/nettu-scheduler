use crate::{shared::mongo_repo::MongoDocument, user::domain::User};

use super::IUserRepo;
use crate::shared::mongo_repo;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct UserRepo {
    collection: Collection,
}

impl UserRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("users"),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for UserRepo {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert::<User, UserMongo>(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<User, UserMongo>(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::find::<User, UserMongo>(&self.collection, &id).await
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::delete::<User, UserMongo>(&self.collection, &id).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMongo {
    _id: String,
    account_id: String,
    external_id: String,
}

impl MongoDocument<User> for UserMongo {
    fn to_domain(&self) -> User {
        User {
            id: self._id.clone(),
            account_id: self.account_id.clone(),
            external_id: self.external_id.clone(),
        }
    }

    fn from_domain(user: &User) -> Self {
        Self {
            _id: user.id.clone(),
            account_id: user.account_id.clone(),
            external_id: user.external_id.clone(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
