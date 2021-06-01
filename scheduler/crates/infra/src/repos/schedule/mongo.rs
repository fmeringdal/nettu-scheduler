use super::IScheduleRepo;
use crate::{
    repos::shared::{
        mongo_repo::{self},
        repo::DeleteResult,
    },
    KVMetadata, MetadataFindQuery,
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

    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule> {
        let filter = doc! {
            "_id": {
                "$in": schedule_ids.iter().map(|id| id.inner_ref()).collect::<Vec<_>>()
            }
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => Vec::new(),
        }
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        match mongo_repo::find_many_by::<_, ScheduleMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => Vec::new(),
        }
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Schedule> {
        mongo_repo::find_by_metadata::<_, ScheduleMongo>(&self.collection, query).await
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
    metadata: Vec<KVMetadata>,
}

impl MongoDocument<Schedule> for ScheduleMongo {
    fn into_domain(self) -> Schedule {
        Schedule {
            id: ID::from(self._id),
            user_id: ID::from(self.user_id),
            account_id: ID::from(self.account_id),
            rules: self.rules,
            timezone: self.timezone.parse().unwrap(),
            metadata: KVMetadata::to_metadata(self.metadata),
        }
    }

    fn from_domain(schedule: &Schedule) -> Self {
        Self {
            _id: schedule.id.inner_ref().clone(),
            user_id: schedule.user_id.inner_ref().clone(),
            account_id: schedule.account_id.inner_ref().clone(),
            rules: schedule.rules.to_owned(),
            timezone: schedule.timezone.to_string(),
            metadata: KVMetadata::new(schedule.metadata.clone()),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": &self._id
        }
    }
}
