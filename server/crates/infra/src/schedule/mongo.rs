use super::IScheduleRepo;
use crate::shared::mongo_repo;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_core::domain::{Schedule, ScheduleRule};
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

    async fn find_by_user(&self, user_id: &str) -> Vec<Schedule> {
        let filter = doc! {
            "user_id": user_id
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
    }

    async fn find_many(&self, schedule_ids: &[String]) -> Vec<Schedule> {
        let ids = schedule_ids
            .iter()
            .map(|id| ObjectId::with_string(id))
            .filter(|id| id.is_ok())
            .map(|id| id.unwrap())
            .collect::<Vec<ObjectId>>();

        let filter = doc! {
            "_id": {
                "$in": ids
            }
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
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
    user_id: String,
    rules: Vec<ScheduleRule>,
    timezone: String,
}

impl MongoDocument<Schedule> for ScheduleMongo {
    fn to_domain(&self) -> Schedule {
        Schedule {
            id: self._id.to_string(),
            user_id: self.user_id.to_string(),
            rules: self.rules.to_owned(),
            timezone: self.timezone.parse().unwrap(),
        }
    }

    fn from_domain(schedule: &Schedule) -> Self {
        Self {
            _id: ObjectId::with_string(&schedule.id).unwrap(),
            user_id: schedule.user_id.to_owned(),
            rules: schedule.rules.to_owned(),
            timezone: schedule.timezone.to_string(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
