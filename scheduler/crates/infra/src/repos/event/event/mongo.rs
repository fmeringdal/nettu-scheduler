use super::IEventRepo;
use crate::repos::shared::mongo_repo;
use crate::repos::shared::query_structs::MetadataFindQuery;
use crate::repos::shared::repo::DeleteResult;
use crate::KVMetadata;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::doc,
    bson::{oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, RRuleOptions, TimeSpan, ID};
use serde::{Deserialize, Serialize};

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
        mongo_repo::insert::<_, CalendarEventMongo>(&self.collection, e).await
    }

    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        mongo_repo::save::<_, CalendarEventMongo>(&self.collection, e).await
    }

    async fn find(&self, event_id: &ID) -> Option<CalendarEvent> {
        let oid = event_id.inner_ref();
        mongo_repo::find::<_, CalendarEventMongo>(&self.collection, &oid).await
    }

    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let mut filter = doc! {
            "calendar_id": calendar_id.inner_ref()
        };
        if let Some(timespan) = timespan {
            filter = doc! {
                "calendar_id": calendar_id.inner_ref(),
                "$and": [
                    {
                        "start_ts": {
                            "$lte": timespan.end()
                        }
                    },
                    {
                        "end_ts": {
                            "$gte": timespan.start()
                        }
                    }
                ]
            };
        }

        mongo_repo::find_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let filter = doc! {
            "_id": {
                "$in": event_ids.iter().map(|id| id.inner_ref()).collect::<Vec<_>>()
            }
        };

        mongo_repo::find_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn delete(&self, event_id: &ID) -> Option<CalendarEvent> {
        let oid = event_id.inner_ref();
        mongo_repo::delete::<_, CalendarEventMongo>(&self.collection, &oid).await
    }

    async fn delete_by_calendar(&self, calendar_id: &ID) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "calendar_id": calendar_id.inner_ref()
        };
        self.collection
            .delete_many(filter, None)
            .await
            .map(|res| DeleteResult {
                deleted_count: res.deleted_count,
            })
            .map_err(anyhow::Error::new)
    }

    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "user_id": user_id.inner_ref()
        };
        mongo_repo::delete_many_by::<_, CalendarEventMongo>(&self.collection, filter).await
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent> {
        mongo_repo::find_by_metadata::<_, CalendarEventMongo>(&self.collection, query).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarEventMongo {
    _id: ObjectId,
    start_ts: i64,
    duration: i64,
    end_ts: i64,
    pub created: i64,
    pub updated: i64,
    busy: bool,
    user_id: ObjectId,
    exdates: Vec<i64>,
    calendar_id: ObjectId,
    account_id: ObjectId,
    recurrence: Option<RRuleOptions>,
    reminder: Option<CalendarEventReminder>,
    is_service: bool,
    metadata: Vec<KVMetadata>,
}

impl MongoDocument<CalendarEvent> for CalendarEventMongo {
    fn to_domain(self) -> CalendarEvent {
        CalendarEvent {
            id: ID::from(self._id),
            start_ts: self.start_ts,
            duration: self.duration,
            end_ts: self.end_ts,
            busy: self.busy,
            created: self.created,
            updated: self.updated,
            user_id: ID::from(self.user_id),
            account_id: ID::from(self.account_id),
            calendar_id: ID::from(self.calendar_id),
            exdates: self.exdates,
            recurrence: self.recurrence,
            reminder: self.reminder,
            is_service: self.is_service,
            metadata: KVMetadata::to_metadata(self.metadata),
        }
    }

    fn from_domain(event: &CalendarEvent) -> Self {
        Self {
            _id: event.id.inner_ref().clone(),
            start_ts: event.start_ts,
            duration: event.duration,
            end_ts: event.end_ts,
            busy: event.busy,
            created: event.created,
            updated: event.updated,
            user_id: event.user_id.inner_ref().clone(),
            account_id: event.account_id.inner_ref().clone(),
            calendar_id: event.calendar_id.inner_ref().clone(),
            exdates: event.exdates.clone(),
            recurrence: event.recurrence.clone(),
            reminder: event.reminder.clone(),
            is_service: event.is_service,
            metadata: KVMetadata::new(event.metadata.clone()),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": &self._id
        }
    }
}
