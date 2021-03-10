use super::IServiceRepo;
use crate::repos::shared::mongo_repo::{self};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Metadata, Service, ServiceResource, TimePlan, ID};
use serde::{Deserialize, Serialize};

pub struct MongoServiceRepo {
    collection: Collection,
}

impl MongoServiceRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("services"),
        }
    }
}

#[async_trait::async_trait]
impl IServiceRepo for MongoServiceRepo {
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        mongo_repo::insert::<_, ServiceMongo>(&self.collection, service).await
    }

    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        mongo_repo::save::<_, ServiceMongo>(&self.collection, service).await
    }

    async fn find(&self, service_id: &ID) -> Option<Service> {
        let oid = service_id.inner_ref();
        mongo_repo::find::<_, ServiceMongo>(&self.collection, &oid).await
    }

    async fn delete(&self, service_id: &ID) -> Option<Service> {
        let oid = service_id.inner_ref();
        mongo_repo::delete::<_, ServiceMongo>(&self.collection, &oid).await
    }

    async fn remove_calendar_from_services(&self, calendar_id: &ID) -> anyhow::Result<()> {
        let calendar_id = calendar_id.as_string();
        let filter = doc! {
            "attributes": {
                "key": "calendars",
                "value": &calendar_id
            }
        };
        let update = doc! {
            "attributes.value": {
                "$pull": &calendar_id
            },
            "users.calendar_ids": {
                "$pull": &calendar_id
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }

    async fn remove_schedule_from_services(&self, schedule_id: &ID) -> anyhow::Result<()> {
        let schedule_id = schedule_id.as_string();
        let filter = doc! {
            "attributes": {
                "key": "schedules",
                "value": &schedule_id
            }
        };
        let update = doc! {
            "attributes.value": {
                "$pull": &schedule_id
            },
            "users.schedule_ids": {
                "$pull": &schedule_id
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }

    async fn remove_user_from_services(&self, user_id: &ID) -> anyhow::Result<()> {
        let user_id = user_id.as_string();
        let filter = doc! {
            "users.user_id": &user_id
        };
        let update = doc! {
            "$pull": {
                "users": {
                    "user_id": &user_id
                }
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResourceMongo {
    pub _id: ObjectId,
    pub user_id: ObjectId,
    pub availibility: TimePlan,
    pub busy: Vec<ObjectId>,
    pub buffer: i64,
    pub closest_booking_time: i64,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentAttribute {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceMongo {
    pub _id: ObjectId,
    pub account_id: ObjectId,
    pub users: Vec<ServiceResourceMongo>,
    pub attributes: Vec<DocumentAttribute>,
    pub metadata: Metadata,
}

impl MongoDocument<Service> for ServiceMongo {
    fn to_domain(&self) -> Service {
        Service {
            id: ID::from(self._id.clone()),
            account_id: ID::from(self.account_id.clone()),
            users: self
                .users
                .iter()
                .map(|user| ServiceResource {
                    id: ID::from(user._id.clone()),
                    user_id: ID::from(user.user_id.clone()),
                    availibility: user.availibility.clone(),
                    busy: user.busy.iter().map(|id| ID::from(id.clone())).collect(),
                    buffer: user.buffer,
                    closest_booking_time: user.closest_booking_time,
                    furthest_booking_time: user.furthest_booking_time,
                })
                .collect(),
            metadata: self.metadata.clone(),
        }
    }

    fn from_domain(service: &Service) -> Self {
        Self {
            _id: service.id.inner_ref().clone(),
            account_id: service.account_id.inner_ref().clone(),
            users: service
                .users
                .iter()
                .map(|user| ServiceResourceMongo {
                    _id: user.id.inner_ref().clone(),
                    user_id: user.user_id.inner_ref().clone(),
                    availibility: user.availibility.clone(),
                    busy: user.busy.iter().map(|id| id.inner_ref().clone()).collect(),
                    buffer: user.buffer,
                    closest_booking_time: user.closest_booking_time,
                    furthest_booking_time: user.furthest_booking_time,
                })
                .collect(),
            metadata: service.metadata.clone(),
            attributes: vec![
                DocumentAttribute {
                    key: "calendars".into(),
                    value: service
                        .users
                        .iter()
                        .map(|u| {
                            u.get_calendar_ids()
                                .iter()
                                .map(|id| id.as_string())
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .collect(),
                },
                DocumentAttribute {
                    key: "schedules".into(),
                    value: service
                        .users
                        .iter()
                        .map(|u| u.get_schedule_id().map(|id| id.as_string()))
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
