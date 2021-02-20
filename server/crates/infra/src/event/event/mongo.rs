use crate::shared::mongo_repo;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::doc,
    bson::{oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_core::ctx::{results::DeleteResult, IReminderRepo};
use nettu_scheduler_core::domain::{CalendarEvent, CalendarView, Reminder};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct EventRepo {
    collection: Collection,
}

impl EventRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-events"),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for EventRepo {
    async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        match mongo_repo::insert::<_, CalendarEventMongo>(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, e: &CalendarEvent) -> Result<(), Box<dyn Error>> {
        match mongo_repo::save::<_, CalendarEventMongo>(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let id = match ObjectId::with_string(event_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::find::<_, CalendarEventMongo>(&self.collection, &id).await
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

        mongo_repo::find_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn find_many(&self, event_ids: &[String]) -> Result<Vec<CalendarEvent>, Box<dyn Error>> {
        let filter = doc! {
            "event_id": {
                "$in": event_ids
            }
        };

        mongo_repo::find_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        let id = match ObjectId::with_string(event_id) {
            Ok(oid) => mongo_repo::MongoPersistenceID::ObjectId(oid),
            Err(_) => return None,
        };
        mongo_repo::delete::<_, CalendarEventMongo>(&self.collection, &id).await
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let filter = doc! {
            "calendar_id": calendar_id
        };
        match self.collection.delete_many(filter, None).await {
            Ok(res) => Ok(DeleteResult {
                deleted_count: res.deleted_count,
            }),
            Err(err) => Err(Box::new(err)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarEventMongo {
    _id: ObjectId,
    start_ts: i64,
    duration: i64,
    end_ts: i64,
    busy: bool,
    user_id: String,
    exdates: Vec<i64>,
    calendar_id: String,
    account_id: String,
    recurrence: Option<RRuleOptions>,
    reminder: Option<CalendarEventReminder>,
}

impl MongoDocument<CalendarEvent> for CalendarEventMongo {
    fn to_domain(&self) -> CalendarEvent {
        CalendarEvent {
            id: self._id.to_string(),
            start_ts: self.start_ts,
            duration: self.duration,
            end_ts: self.end_ts,
            busy: self.busy,
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            exdates: self.exdates.clone(),
            calendar_id: self.calendar_id.clone(),
            recurrence: self.recurrence.clone(),
            reminder: self.reminder.clone(),
        }
    }

    fn from_domain(event: &CalendarEvent) -> Self {
        Self {
            _id: ObjectId::with_string(&event.id).unwrap(),
            start_ts: event.start_ts,
            duration: event.duration,
            end_ts: event.end_ts,
            busy: event.busy,
            user_id: event.user_id.clone(),
            account_id: event.account_id.clone(),
            exdates: event.exdates.clone(),
            calendar_id: event.calendar_id.clone(),
            recurrence: event.recurrence.clone(),
            reminder: event.reminder.clone(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
