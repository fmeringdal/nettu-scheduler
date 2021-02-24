use super::IServiceRepo;
use crate::{schedule, shared::mongo_repo};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_core::{Plan, Service, ServiceResource};
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
        match mongo_repo::insert::<_, ServiceMongo>(&self.collection, service).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, service: &Service) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<_, ServiceMongo>(&self.collection, service).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, service_id: &str) -> Option<Service> {
        let id = match ObjectId::with_string(service_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find::<_, ServiceMongo>(&self.collection, &id).await
    }

    async fn delete(&self, service_id: &str) -> Option<Service> {
        let id = match ObjectId::with_string(service_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete::<_, ServiceMongo>(&self.collection, &id).await
    }

    async fn remove_calendar_from_services(&self, calendar_id: &str) -> Result<(), Box<dyn Error>> {
        let filter = doc! {
            "attributes": {
                "key": "calendars",
                "value": calendar_id
            }
        };
        let update = doc! {
            "attributes.value": {
                "$pull": calendar_id
            },
            "users.calendar_ids": {
                "$pull": calendar_id
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }

    async fn remove_schedule_from_services(&self, schedule_id: &str) -> Result<(), Box<dyn Error>> {
        let filter = doc! {
            "attributes": {
                "key": "schedules",
                "value": schedule_id
            }
        };
        let update = doc! {
            "attributes.value": {
                "$pull": schedule_id
            },
            "users.schedule_ids": {
                "$pull": schedule_id
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResourceMongo {
    pub _id: ObjectId,
    pub user_id: String,
    pub availibility: Plan,
    pub busy: Vec<String>,
    pub buffer: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentAttribute {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceMongo {
    pub _id: ObjectId,
    pub account_id: String,
    pub users: Vec<ServiceResourceMongo>,
    pub attributes: Vec<DocumentAttribute>,
}

impl MongoDocument<Service> for ServiceMongo {
    fn to_domain(&self) -> Service {
        Service {
            id: self._id.to_string(),
            account_id: self.account_id.clone(),
            users: self
                .users
                .iter()
                .map(|user| ServiceResource {
                    id: user._id.to_string(),
                    user_id: user.user_id.clone(),
                    availibility: user.availibility.clone(),
                    busy: user.busy.clone(),
                    buffer: user.buffer as usize,
                })
                .collect(),
        }
    }

    fn from_domain(service: &Service) -> Self {
        Self {
            _id: ObjectId::with_string(&service.id).unwrap(),
            account_id: service.account_id.clone(),
            users: service
                .users
                .iter()
                .map(|user| ServiceResourceMongo {
                    _id: ObjectId::with_string(&user.id).unwrap(),
                    user_id: user.user_id.clone(),
                    availibility: user.availibility.clone(),
                    busy: user.busy.clone(),
                    buffer: user.buffer as i64,
                })
                .collect(),
            attributes: vec![
                DocumentAttribute {
                    key: "calendars".into(),
                    value: service
                        .users
                        .iter()
                        .map(|u| u.get_calendar_ids())
                        .flatten()
                        .collect(),
                },
                DocumentAttribute {
                    key: "schedules".into(),
                    value: service
                        .users
                        .iter()
                        .map(|u| u.get_schedule_id())
                        .filter(|schedule| schedule.is_some())
                        .map(|schedule| schedule.unwrap())
                        .collect(),
                },
            ],
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
