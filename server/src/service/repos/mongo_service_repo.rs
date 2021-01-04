use crate::service::domain::{Service, ServiceResource};

use super::IServiceRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;


pub struct ServiceRepo {
    collection: Collection,
}

impl ServiceRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("services"),
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

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResourceMongo {
    pub _id: ObjectId,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceMongo {
    pub _id: ObjectId,
    pub account_id: String,
    pub users: Vec<ServiceResourceMongo>,
}

impl ServiceMongo {
    pub fn to_domain(&self) -> Service {
        Service {
            id: self._id.to_string(),
            account_id: self.account_id.clone(),
            users: self
                .users
                .iter()
                .map(|user| ServiceResource {
                    id: user._id.to_string(),
                    user_id: user.user_id.clone(),
                    calendar_ids: user.calendar_ids.clone(),
                })
                .collect(),
        }
    }

    pub fn from_domain(service: &Service) -> Self {
        Self {
            _id: ObjectId::with_string(&service.id).unwrap(),
            account_id: service.account_id.clone(),
            users: service
                .users
                .iter()
                .map(|user| ServiceResourceMongo {
                    _id: ObjectId::with_string(&user.id).unwrap(),
                    user_id: user.user_id.clone(),
                    calendar_ids: user.calendar_ids.clone(),
                })
                .collect(),
        }
    }
}

impl MongoPersistence for Service {
    fn to_domain(doc: Document) -> Self {
        let doc: ServiceMongo = from_bson(Bson::Document(doc)).unwrap();
        doc.to_domain()
    }

    fn to_persistence(&self) -> Document {
        let doc = ServiceMongo::from_domain(self);
        to_bson(&doc).unwrap().as_document().unwrap().to_owned()
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
