use crate::calendar::domain::calendar::Calendar;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

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
        let coll = self.collection.read().await;
        let _res = coll.insert_one(to_persistence(calendar), None).await;
        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let filter = doc! {
            "_id": ObjectId::with_string(&calendar.id)?
        };
        let _res = coll
            .update_one(filter, to_persistence(calendar), None)
            .await;
        Ok(())
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
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

    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar> {
        let filter = doc! {
            "user_id": user_id
        };
        let coll = self.collection.read().await;
        let res = coll.find(filter, None).await;
        match res {
            Ok(mut cursor) => {
                let mut calendars = vec![];

                while let Some(result) = cursor.next().await {
                    match result {
                        Ok(document) => {
                            calendars.push(to_domain(document));
                        }
                        Err(e) => {
                            println!("Error getting cursor calendar: {:?}", e);
                        }
                    }
                }

                calendars
            }
            _ => vec![],
        }
    }

    async fn delete(&self, calendar_id: &str) -> Option<Calendar> {
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
        "user_id": calendar.external_user_id.clone(),
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
        external_user_id: from_bson(raw.get("user_id").unwrap().clone()).unwrap(),
    };

    calendar
}
