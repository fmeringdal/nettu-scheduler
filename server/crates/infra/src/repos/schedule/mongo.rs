use super::IScheduleRepo;
use crate::repos::shared::{
    mongo_repo::{self, create_object_id},
    repo::DeleteResult,
};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Schedule, ScheduleRule};
use serde::{Deserialize, Serialize};

pub struct MongoScheduleRepo {
    collection: Collection,
}

impl MongoScheduleRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("schedules"),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for MongoScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()> {
        mongo_repo::insert::<_, ScheduleMongo>(&self.collection, schedule).await
    }

    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()> {
        mongo_repo::save::<_, ScheduleMongo>(&self.collection, schedule).await
    }

    async fn find(&self, schedule_id: &str) -> Option<Schedule> {
        let oid = create_object_id(schedule_id)?;
        mongo_repo::find::<_, ScheduleMongo>(&self.collection, &oid).await
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
        let oid = create_object_id(schedule_id)?;
        mongo_repo::delete::<_, ScheduleMongo>(&self.collection, &oid).await
    }

    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "user_id": user_id
        };
        mongo_repo::delete_many_by::<_, ScheduleMongo>(&self.collection, filter).await
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
