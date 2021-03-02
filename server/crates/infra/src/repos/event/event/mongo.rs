use super::IEventRepo;
use crate::repos::shared::mongo_repo::{self, create_object_id};
use crate::repos::shared::repo::DeleteResult;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::doc,
    bson::{oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, CalendarView, RRuleOptions};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct MongoEventRepo {
    collection: Collection,
}

impl MongoEventRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-events"),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for MongoEventRepo {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        match mongo_repo::insert::<_, CalendarEventMongo>(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        match mongo_repo::save::<_, CalendarEventMongo>(&self.collection, e).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
        let oid = create_object_id(event_id)?;
        mongo_repo::find::<_, CalendarEventMongo>(&self.collection, &oid).await
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &str,
        view: Option<&CalendarView>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
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

    async fn find_many(&self, event_ids: &[String]) -> anyhow::Result<Vec<CalendarEvent>> {
        let filter = doc! {
            "event_id": {
                "$in": event_ids
            }
        };

        mongo_repo::find_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn delete(&self, event_id: &str) -> Option<CalendarEvent> {
        let oid = create_object_id(event_id)?;
        mongo_repo::delete::<_, CalendarEventMongo>(&self.collection, &oid).await
    }

    async fn delete_by_calendar(&self, calendar_id: &str) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "calendar_id": calendar_id
        };
        self.collection
            .delete_many(filter, None)
            .await
            .map(|res| DeleteResult {
                deleted_count: res.deleted_count,
            })
            .map_err(anyhow::Error::new)
    }

    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "user_id": user_id
        };
        mongo_repo::delete_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
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
    services: Vec<String>,
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
            services: self.services.clone(),
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
            services: event.services.clone(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
