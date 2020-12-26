use crate::event::domain::event::CalendarEvent;
use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Bson::{Int64};
use mongodb::bson::{doc, from_bson, to_bson, Document};
use mongodb::Collection;
use mongodb::Database;

#[async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>>;
    async fn find(&self, event_id: &str) -> Option<CalendarEvent>;
}

pub struct EventRepo {
    collection: Collection,
}

unsafe impl Send for EventRepo {}
unsafe impl Sync for EventRepo {}

impl EventRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-events"),
        }
    }

    pub async fn get_all(&self) -> Vec<String> {
        let r = self.collection.find(None, None).await;
        vec![]
    }
}

#[async_trait]
impl IEventRepo for EventRepo {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.collection.insert_one(to_persistence(e), None).await;
        Ok(())
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let filter = doc! {
            "_id": ObjectId::with_string(event_id).unwrap()
        };
        let res = self.collection.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let event = to_domain(doc.unwrap());
                Some(event)
            }
            _ => None,
        }
    }
}

fn to_persistence(e: &CalendarEvent) -> Document {
    let max_timestamp = 9999999999;

    let mut d = doc! {
        "start_ts": Int64(e.start_ts),
        "duration": Int64(e.duration),
        "end_ts": Int64(e.end_ts.unwrap_or(max_timestamp)),
        "user_id": e.user_id,
        "calendar_id": e.calendar_id,
    };
    if let Some(recurrence) = &e.recurrence {
        d.insert("recurrence", to_bson(recurrence).unwrap());
    }
    d
}

fn to_domain(raw: Document) -> CalendarEvent {
    let mut e = CalendarEvent {
        start_ts: from_bson(raw.get("start_ts").unwrap().clone()).unwrap(),
        duration: from_bson(raw.get("duration").unwrap().clone()).unwrap(),
        recurrence: None,
        end_ts: None,
        exdates: vec![],
        calendar_id: String::from("1"),
        user_id: String::from("2"),
    };
    if let Some(rrule_opts_bson) = raw.get("recurrence") {
        e.set_reccurrence(from_bson(rrule_opts_bson.clone()).unwrap());
    };
    e
}
