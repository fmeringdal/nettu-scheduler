use super::ICalendarRepo;
use crate::calendar::domain::calendar::Calendar;
use crate::shared::mongo_repo;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::RwLock;

pub struct CalendarRepo {
    collection: Collection,
}

impl CalendarRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendars"),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for CalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert(&self.collection, calendar).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, calendar).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
        let id = match ObjectId::with_string(calendar_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find(&self.collection, &id).await
    }

    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar> {
        let filter = doc! {
            "user_id": user_id
        };
        match mongo_repo::find_many_by(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![],
        }
    }

    async fn delete(&self, calendar_id: &str) -> Option<Calendar> {
        let id = match ObjectId::with_string(calendar_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete(&self.collection, &id).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarMongo {
    _id: ObjectId,
    user_id: String,
}

impl CalendarMongo {
    pub fn to_domain(&self) -> Calendar {
        Calendar {
            id: self._id.to_string(),
            user_id: self.user_id.clone(),
        }
    }

    pub fn from_domain(calendar: &Calendar) -> Self {
        Self {
            _id: ObjectId::with_string(&calendar.id).unwrap(),
            user_id: calendar.user_id.clone(),
        }
    }
}

impl MongoPersistence for Calendar {
    fn to_domain(doc: Document) -> Self {
        let doc: CalendarMongo = from_bson(Bson::Document(doc)).unwrap();
        doc.to_domain()
    }

    fn to_persistence(&self) -> Document {
        let doc = CalendarMongo::from_domain(self);
        to_bson(&doc).unwrap().as_document().unwrap().to_owned()
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
