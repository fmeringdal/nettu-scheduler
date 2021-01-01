use mongo_repo::MongoPersistence;
use mongodb::{
    bson::doc,
    bson::{from_bson, oid::ObjectId, to_bson, Bson, Document},
    Collection, Database,
};
use tokio::sync::RwLock;

use super::repos::{DeleteResult, IEventRepo};
use crate::shared::mongo_repo;
use crate::{calendar::domain::calendar_view::CalendarView, event::domain::event::CalendarEvent};
use std::error::Error;

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

#[async_trait::async_trait]
impl IEventRepo for EventRepo {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let id = match ObjectId::with_string(event_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find(&self.collection, &id).await
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let mut filter = doc! {
            "calendar_id": calendar_id
        };
        if let Some(view) = view {
            filter = doc! {
                "calendar_id": calendar_id,
                "$and": [
                    {
                        "start_ts": {
                            "$lte": view.get_end()
                        }
                    },
                    {
                        "end_ts": {
                            "$gte": view.get_start()
                        }
                    }
                ]
            };
        }

        mongo_repo::find_many_by(&self.collection, filter).await
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        let id = match ObjectId::with_string(event_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete(&self.collection, &id).await
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let filter = doc! {
            "calendar_id": calendar_id
        };
        let coll = self.collection.read().await;
        match coll.delete_many(filter, None).await {
            Ok(res) => Ok(DeleteResult {
                deleted_count: res.deleted_count,
            }),
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl MongoPersistence for CalendarEvent {
    fn to_domain(doc: Document) -> Self {
        let id = match doc.get("_id").unwrap() {
            Bson::ObjectId(oid) => oid.to_string(),
            _ => unreachable!("This should not happen"),
        };

        let mut e = CalendarEvent {
            id,
            start_ts: from_bson(doc.get("start_ts").unwrap().clone()).unwrap(),
            duration: from_bson(doc.get("duration").unwrap().clone()).unwrap(),
            recurrence: from_bson(doc.get("recurrence").unwrap().clone()).unwrap(),
            end_ts: from_bson(doc.get("end_ts").unwrap().clone()).unwrap(),
            busy: from_bson(doc.get("busy").unwrap().clone()).unwrap(),
            exdates: from_bson(doc.get("exdates").unwrap().clone()).unwrap(),
            calendar_id: from_bson(doc.get("calendar_id").unwrap().clone()).unwrap(),
            user_id: from_bson(doc.get("user_id").unwrap().clone()).unwrap(),
        };

        if let Some(rrule_opts_bson) = doc.get("recurrence") {
            e.set_reccurrence(from_bson(rrule_opts_bson.clone()).unwrap(), false);
        };
        e
    }

    fn to_persistence(&self) -> Document {
        let max_timestamp = 9999999999;

        let mut d = doc! {
            "_id": ObjectId::with_string(&self.id).unwrap(),
            "start_ts": Bson::Int64(self.start_ts),
            "duration": Bson::Int64(self.duration),
            "busy": Bson::Boolean(self.busy),
            "end_ts": Bson::Int64(self.end_ts.unwrap_or(max_timestamp)),
            "user_id": self.user_id.clone(),
            "exdates": self.exdates.clone(),
            "calendar_id": self.calendar_id.clone(),
        };
        if let Some(recurrence) = &self.recurrence {
            d.insert("recurrence", to_bson(recurrence).unwrap());
        }
        d
    }

    fn get_persistence_id(&self) -> anyhow::Result<mongo_repo::MongoPersistenceID> {
        let oid = ObjectId::with_string(&self.id)?;
        Ok(mongo_repo::MongoPersistenceID::ObjectId(oid))
    }
}
