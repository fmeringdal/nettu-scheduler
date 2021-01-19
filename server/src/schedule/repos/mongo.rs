use super::IScheduleRepo;
use crate::schedule::domain::{Schedule, ScheduleRule};
use crate::shared::mongo_repo;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct ScheduleRepo {
    collection: Collection,
}

impl ScheduleRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("schedules"),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for ScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert::<_, ScheduleMongo>(&self.collection, schedule).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<_, ScheduleMongo>(&self.collection, schedule).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, schedule_id: &str) -> Option<Schedule> {
        let id = match ObjectId::with_string(schedule_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find::<_, ScheduleMongo>(&self.collection, &id).await
    }

    async fn delete(&self, schedule_id: &str) -> Option<Schedule> {
        let id = match ObjectId::with_string(schedule_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete::<_, ScheduleMongo>(&self.collection, &id).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ScheduleMongo {
    _id: ObjectId,
    rules: Vec<ScheduleRule>,
    timezone: String,
}

impl MongoDocument<Schedule> for ScheduleMongo {
    fn to_domain(&self) -> Schedule {
        Schedule {
            id: self._id.to_string(),
            rules: self.rules.to_owned(),
            timezone: self.timezone.to_owned() 
        }
    }

    fn from_domain(schedule: &Schedule) -> Self {
        Self {
            _id: ObjectId::with_string(&schedule.id).unwrap(),
            rules: schedule.rules.to_owned(),
            timezone: schedule.timezone.to_owned()
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
