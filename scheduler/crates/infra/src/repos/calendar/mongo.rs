use super::ICalendarRepo;
use crate::repos::shared::{
    mongo_repo::{self, MongoMetadata},
    query_structs::MetadataFindQuery,
    repo::DeleteResult,
};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{Calendar, CalendarSettings, Metadata, ID};
use serde::{Deserialize, Serialize};

pub struct MongoCalendarRepo {
    collection: Collection,
}

impl MongoCalendarRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendars"),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for MongoCalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()> {
        mongo_repo::insert::<_, CalendarMongo>(&self.collection, calendar).await
    }

    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()> {
        mongo_repo::save::<_, CalendarMongo>(&self.collection, calendar).await
    }

    async fn find(&self, calendar_id: &ID) -> Option<Calendar> {
        let oid = calendar_id.inner_ref();
        mongo_repo::find::<_, CalendarMongo>(&self.collection, &oid).await
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        match mongo_repo::find_many_by::<_, CalendarMongo>(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
    }

    async fn delete(&self, calendar_id: &ID) -> Option<Calendar> {
        let oid = calendar_id.inner_ref();
        mongo_repo::delete::<_, CalendarMongo>(&self.collection, &oid).await
    }

    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        mongo_repo::delete_many_by::<_, CalendarMongo>(&self.collection, filter).await
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Calendar> {
        mongo_repo::find_by_metadata::<_, CalendarMongo>(&self.collection, query).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarMongo {
    _id: ObjectId,
    user_id: ObjectId,
    account_id: ObjectId,
    settings: CalendarSettingsMongo,
    metadata: Vec<MongoMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarSettingsMongo {
    week_start: isize,
    timezone: String,
}

impl MongoDocument<Calendar> for CalendarMongo {
    fn to_domain(self) -> Calendar {
        Calendar {
            id: ID::from(self._id),
            user_id: ID::from(self.user_id),
            account_id: ID::from(self.account_id),
            settings: CalendarSettings {
                week_start: self.settings.week_start,
                timezone: self.settings.timezone.parse().unwrap(),
            },
            metadata: MongoMetadata::to_metadata(self.metadata),
        }
    }

    fn from_domain(calendar: &Calendar) -> Self {
        Self {
            _id: calendar.id.inner_ref().clone(),
            user_id: calendar.user_id.inner_ref().clone(),
            account_id: calendar.account_id.inner_ref().clone(),
            settings: CalendarSettingsMongo {
                week_start: calendar.settings.week_start,
                timezone: calendar.settings.timezone.to_string(),
            },
            metadata: MongoMetadata::new(calendar.metadata.clone()),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": &self._id
        }
    }
}
