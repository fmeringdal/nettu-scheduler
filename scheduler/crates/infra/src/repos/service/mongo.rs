use super::IServiceRepo;
use crate::{
    repos::shared::{
        mongo_repo::{self},
        query_structs::MetadataFindQuery,
    },
    KVMetadata,
};
use futures::StreamExt;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Service, ServiceResource, TimePlan, ID};
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
            "$pull": {
                "attributes": {
                    "value": &calendar_id
                },
                "users": {
                    "availibility": {
                        "id": &calendar_id
                    },
                    "busy": &calendar_id
                }
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
            "$pull": {
                "attributes": {
                    "value": &schedule_id
                },
                "users": {
                    "availibility": {
                        "id": schedule_id
                    }
                }
            }
        };
        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }

    async fn remove_user_from_services(&self, user_id: &ID) -> anyhow::Result<()> {
        let user_id = user_id.as_string();
        let filter = doc! {
            "attributes": {
                "key": "users",
                "value": &user_id
            }
        };
        let update = doc! {
            "$pull": {
                "attributes": {
                    "value": &user_id
                },
                "users": {
                    "user_id": &user_id
                }
            }
        };
        // println!("ooooooooooooooooooooooooooooooooooooooooooooooooooook");
        // println!("Filter: {:?}", filter);
        // println!("Update: {:?}", update);
        // let d = doc! {};
        // let mut cursor = self.collection.find(d, None).await.unwrap();
        // let mut all = vec![];
        // while let Some(result) = cursor.next().await {
        //     match result {
        //         Ok(document) => {
        //             all.push(document);
        //         }
        //         Err(e) => {}
        //     }
        // }
        // println!("Alll: {:?}", all);
        // println!("ooooooooooooooooooooooooooooooooooooooooooooooooooook done");

        mongo_repo::update_many::<_, ServiceMongo>(&self.collection, filter, update).await
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service> {
        mongo_repo::find_by_metadata::<_, ServiceMongo>(&self.collection, query).await
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
    pub metadata: Vec<KVMetadata>,
}

impl MongoDocument<Service> for ServiceMongo {
    fn to_domain(self) -> Service {
        Service {
            id: ID::from(self._id),
            account_id: ID::from(self.account_id),
            users: self
                .users
                .into_iter()
                .map(|user| ServiceResource {
                    id: ID::from(user._id),
                    user_id: ID::from(user.user_id),
                    availibility: user.availibility,
                    busy: user.busy.into_iter().map(ID::from).collect(),
                    buffer: user.buffer,
                    closest_booking_time: user.closest_booking_time,
                    furthest_booking_time: user.furthest_booking_time,
                })
                .collect(),
            metadata: KVMetadata::to_metadata(self.metadata),
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
            metadata: KVMetadata::new(service.metadata.clone()),
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
                DocumentAttribute {
                    key: "users".into(),
                    value: service
                        .users
                        .iter()
                        .map(|u| u.user_id.as_string())
                        .collect(),
                },
            ],
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": &self._id
        }
    }
}
