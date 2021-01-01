use crate::calendar::domain::calendar::Calendar;
use mongo_repo::MongoPersistence;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;
use crate::shared::mongo_repo;
use super::ICalendarRepo;

pub struct CalendarRepo {
    collection: RwLock<Collection>,
}

// RwLock is Send + Sync
unsafe impl Send for CalendarRepo {}
unsafe impl Sync for CalendarRepo {}

impl CalendarRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("calendars")),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for CalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert(&self.collection, calendar).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()) // fix this
        }
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, calendar).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()) // fix this
        }
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
        let id = match ObjectId::with_string(calendar_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None
        };
        mongo_repo::find(&self.collection, &id).await
    }

    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar> {
        let filter = doc! {
            "user_id": user_id
        };
        match mongo_repo::find_many_by(&self.collection, filter).await {
            Ok(cals) => cals,
            Err(_) => vec![]
        }
    }

    async fn delete(&self, calendar_id: &str) -> Option<Calendar> {
        let id = match ObjectId::with_string(calendar_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None
        };
        mongo_repo::delete(&self.collection, &id).await
    }
}

impl MongoPersistence for Calendar {
    fn to_domain(doc: Document) -> Self {
        let id = match doc.get("_id").unwrap() {
            Bson::ObjectId(oid) => oid.to_string(),
            _ => unreachable!("This should not happen"),
        };
    
        let calendar = Calendar {
            id,
            user_id: from_bson(doc.get("user_id").unwrap().clone()).unwrap(),
        };
    
        calendar
    }

    fn to_persistence(&self) -> Document {
        let raw = doc! {
            "_id": ObjectId::with_string(&self.id).unwrap(),
            "user_id": self.user_id.clone(),
        };
    
        raw
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}