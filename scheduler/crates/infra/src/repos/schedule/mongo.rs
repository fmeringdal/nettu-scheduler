use super::IScheduleRepo;
use crate::repos::shared::{
    mongo_repo::{self},
    repo::DeleteResult,
};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Schedule, ScheduleRule, ID};
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

    async fn find(&self, schedule_id: &ID) -> Option<Schedule> {
        let oid = schedule_id.inner_ref();
        mongo_repo::find::<_, ScheduleMongo>(&self.collection, &oid).await
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
    }

    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule> {
        let filter = doc! {
            "_id": {
                "$in": schedule_ids.iter().map(|id| id.inner_ref()).collect::<Vec<_>>()
            }
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
    }

    async fn delete(&self, schedule_id: &ID) -> Option<Schedule> {
        let oid = schedule_id.inner_ref();
        mongo_repo::delete::<_, ScheduleMongo>(&self.collection, &oid).await
    }

    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        mongo_repo::delete_many_by::<_, ScheduleMongo>(&self.collection, filter).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ScheduleMongo {
    _id: ObjectId,
    user_id: ObjectId,
    account_id: ObjectId,
    rules: Vec<ScheduleRule>,
    timezone: String,
}

impl MongoDocument<Schedule> for ScheduleMongo {
    fn to_domain(&self) -> Schedule {
        Schedule {
            id: ID::from(self._id.clone()),
            user_id: ID::from(self.user_id.clone()),
            account_id: ID::from(self.account_id.clone()),
            rules: self.rules.to_owned(),
            timezone: self.timezone.parse().unwrap(),
        }
    }

    fn from_domain(schedule: &Schedule) -> Self {
        Self {
            _id: schedule.id.inner_ref().clone(),
            user_id: schedule.user_id.inner_ref().clone(),
            account_id: schedule.account_id.inner_ref().clone(),
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
