use super::calendar::Calendar;
use super::event::CalendarEvent;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Bson::Int64;
use mongodb::bson::{doc, from_bson, to_bson, Document, Timestamp};
use mongodb::options::FindOptions;
use mongodb::results::*;
use mongodb::Collection;
use mongodb::Database;

pub trait IEventRepo {
    fn create(&self, e: Calendar) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct EventRepo {
    collection: Collection,
}

impl EventRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-events"),
        }
    }

    pub async fn insert(&self, e: &CalendarEvent) -> () {
        self.collection.insert_one(to_persistence(e), None).await;
    }

    pub async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
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

    pub async fn get_all(&self) -> Vec<String> {
        let r = self.collection.find(None, None).await;
        vec![]
    }
}

fn to_persistence(e: &CalendarEvent) -> Document {
    let mut d = doc! {
        "start_ts": Int64(e.start_ts),
        "duration": Int64(e.duration),
        "end_ts": Int64(e.end_ts.unwrap_or(999999999)),
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

impl IEventRepo for EventRepo {
    fn create(&self, e: Calendar) -> Result<(), Box<dyn std::error::Error>> {
        //self.collection.insert_one(to_persistence(e), None);

        Ok(())
    }
}
