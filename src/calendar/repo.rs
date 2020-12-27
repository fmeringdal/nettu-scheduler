use crate::event::domain::calendar::Calendar;
use async_trait::async_trait;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Bson::Int64, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

#[async_trait]
pub trait ICalendarRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn delete(&self, event_id: &str) -> Option<CalendarEvent>;
}

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

#[async_trait]
impl ICalendarRepo for CalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let res = coll.insert_one(to_persistence(calendar), None).await;
        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let filter = doc! {
            "_id": ObjectId::with_string(&e.id)?
        };
        let res = coll.update_one(filter, to_persistence(calendar), None).await;
        Ok(())
    }

    async fn find(&self, calendar_id: &str) -> Option<CalendarEvent> {
        let filter = doc! {
            "_id": ObjectId::with_string(calendar_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let calendar = to_domain(doc.unwrap());
                Some(calendar)
            }
            _ => None,
        }
    }

    async fn delete(&self, calendar_id: &str) -> Option<CalendarEvent> {
        let filter = doc! {
            "_id": ObjectId::with_string(calendar_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one_and_delete(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let calendar = to_domain(doc.unwrap());
                Some(calendar)
            }
            _ => None,
        }
    }
}

fn to_persistence(calendar: &Calendar) -> Document {

    let raw = doc! {
        "_id": ObjectId::with_string(&calendar.id).unwrap(),
        "user_id": e.user_id.clone(),
    };
    
    raw
}

fn to_domain(raw: Document) -> Calendar {
    let id = match raw.get("_id").unwrap() {
        Bson::ObjectId(oid) => oid.to_string(),
        _ => unreachable!("This should not happen"),
    };

    let calendar = Calendar {
        id,
        user_id: from_bson(raw.get("user_id").unwrap().clone()).unwrap(),
    };
     
    calendar
}
