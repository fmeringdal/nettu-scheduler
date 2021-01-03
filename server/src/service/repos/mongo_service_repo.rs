use crate::service::domain::{Service, ServiceResource};

use super::IServiceRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Bson::Array, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

pub struct ServiceRepo {
    collection: RwLock<Collection>,
}

impl ServiceRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("services")),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for ServiceRepo {
    async fn insert(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert(&self.collection, service).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, service).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, service_id: &str) -> Option<Service> {
        let id = match ObjectId::with_string(service_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find(&self.collection, &id).await
    }

    async fn delete(&self, service_id: &str) -> Option<Service> {
        let id = match ObjectId::with_string(service_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete(&self.collection, &id).await
    }
}

impl MongoPersistence for Service {
    fn to_domain(doc: Document) -> Self {
        let id = match doc.get("_id").unwrap() {
            Bson::ObjectId(oid) => oid.to_string(),
            _ => unreachable!("This should not happen"),
        };

        Service {
            id,
            account_id: from_bson(doc.get("account_id").unwrap().to_owned()).unwrap(),
            users: doc
                .get("users")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|u| ServiceResource::to_domain(u.as_document().unwrap().to_owned()))
                .collect(),
        }
    }

    fn to_persistence(&self) -> Document {
        let raw = doc! {
            "_id": ObjectId::with_string(&self.id).unwrap(),
            "account_id": to_bson(&self.account_id).unwrap(),
            "users": self.users
                .iter()
                .map(|u| to_bson(&u.to_persistence()).unwrap())
                .collect::<Vec<_>>()
        };

        raw
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}

impl MongoPersistence for ServiceResource {
    fn to_domain(doc: Document) -> Self {
        let id = match doc.get("_id").unwrap() {
            Bson::ObjectId(oid) => oid.to_string(),
            _ => unreachable!("This should not happen"),
        };

        Self {
            id,
            user_id: from_bson(doc.get("user_id").unwrap().to_owned()).unwrap(),
            calendar_ids: from_bson(doc.get("calendar_ids").unwrap().to_owned()).unwrap(),
        }
    }

    fn to_persistence(&self) -> Document {
        let raw = doc! {
            "_id": ObjectId::with_string(&self.id).unwrap(),
            "user_id": to_bson(&self.user_id).unwrap(),
            "calendar_ids": to_bson(&self.calendar_ids).unwrap(),
        };

        raw
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
