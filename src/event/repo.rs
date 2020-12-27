use crate::event::domain::event::CalendarEvent;
use async_trait::async_trait;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_bson, Bson, Bson::Int64, Document},
    results::DeleteResult,
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

#[async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn find_by_calendar(
        &self,
        calendar_id: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>>;
    async fn delete(&self, event_id: &str) -> Option<CalendarEvent>;
    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>>;
}

pub struct EventRepo {
    collection: RwLock<Collection>,
}

// RwLock is Send + Sync
unsafe impl Send for EventRepo {}
unsafe impl Sync for EventRepo {}

impl EventRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("calendar-events")),
        }
    }
}

#[async_trait]
impl IEventRepo for EventRepo {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let res = coll.insert_one(to_persistence(e), None).await;
        Ok(())
    }

    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        println!("Id i got: {}", e.id);
        println!("Update: {:?}", e);
        let filter = doc! {
            "_id": ObjectId::with_string(&e.id)?
        };
        println!("Update beofre");
        let res = coll.update_one(filter, to_persistence(e), None).await;
        println!("Update res: {:?}", res);
        Ok(())
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let filter = doc! {
            "_id": ObjectId::with_string(event_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let event = to_domain(doc.unwrap());
                Some(event)
            }
            _ => None,
        }
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let filter = doc! {
            "calendar_id": calendar_id
        };
        let coll = self.collection.read().await;
        let res = coll.find(filter, None).await;

        match res {
            Ok(mut cursor) => {
                let mut events = vec![];
                while let Some(result) = cursor.next().await {
                    match result {
                        Ok(document) => {
                            events.push(to_domain(document));
                        }
                        Err(e) => {
                            println!("Error getting cursor calendar event: {:?}", e);
                        }
                    }
                }

                Ok(events)
            }
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        let filter = doc! {
            "_id": ObjectId::with_string(event_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one_and_delete(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let event = to_domain(doc.unwrap());
                Some(event)
            }
            _ => None,
        }
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let filter = doc! {
            "calendar_id": calendar_id
        };
        let coll = self.collection.read().await;
        match coll.delete_many(filter, None).await {
            Ok(res) => Ok(res),
            Err(err) => Err(Box::new(err)),
        }
    }
}

fn to_persistence(e: &CalendarEvent) -> Document {
    let max_timestamp = 9999999999;

    let mut d = doc! {
        "_id": ObjectId::with_string(&e.id).unwrap(),
        "start_ts": Int64(e.start_ts),
        "duration": Int64(e.duration),
        "end_ts": Int64(e.end_ts.unwrap_or(max_timestamp)),
        "user_id": e.user_id.clone(),
        "calendar_id": e.calendar_id.clone(),
    };
    if let Some(recurrence) = &e.recurrence {
        d.insert("recurrence", to_bson(recurrence).unwrap());
    }
    d
}

fn to_domain(raw: Document) -> CalendarEvent {
    let id = match raw.get("_id").unwrap() {
        Bson::ObjectId(oid) => oid.to_string(),
        _ => unreachable!("This should not happen"),
    };

    let mut e = CalendarEvent {
        id,
        start_ts: from_bson(raw.get("start_ts").unwrap().clone()).unwrap(),
        duration: from_bson(raw.get("duration").unwrap().clone()).unwrap(),
        recurrence: None,
        end_ts: None,
        exdates: vec![],
        calendar_id: String::from("1"),
        user_id: String::from("2"),
    };
    if let Some(rrule_opts_bson) = raw.get("recurrence") {
        e.set_reccurrence(from_bson(rrule_opts_bson.clone()).unwrap(), false);
    };
    e
}
