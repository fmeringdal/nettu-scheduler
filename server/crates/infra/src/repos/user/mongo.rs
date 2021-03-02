use super::IUserRepo;
use crate::repos::shared::mongo_repo;
use crate::repos::shared::mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::User;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct MongoUserRepo {
    collection: Collection,
}

impl MongoUserRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("users"),
        }
    }
}

#[async_trait::async_trait]
impl IUserRepo for MongoUserRepo {
    async fn insert(&self, user: &User) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert::<_, UserMongo>(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<_, UserMongo>(&self.collection, user).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::find::<_, UserMongo>(&self.collection, &id).await
    }

    async fn delete(&self, user_id: &str) -> Option<User> {
        let id = mongo_repo::MongoPersistenceID::String(String::from(user_id));
        mongo_repo::delete::<_, UserMongo>(&self.collection, &id).await
    }

    async fn find_by_account_id(&self, user_id: &str, account_id: &str) -> Option<User> {
        let filter = doc! {
            "_id": ObjectId::with_string(user_id).unwrap(),
            "account_id": account_id
        };
        mongo_repo::find_one_by::<_, UserMongo>(&self.collection, filter).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMongo {
    _id: String,
    account_id: String,
}

impl MongoDocument<User> for UserMongo {
    fn to_domain(&self) -> User {
        User {
            id: self._id.clone(),
            account_id: self.account_id.clone(),
        }
    }

    fn from_domain(user: &User) -> Self {
        Self {
            _id: user.id.clone(),
            account_id: user.account_id.clone(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
