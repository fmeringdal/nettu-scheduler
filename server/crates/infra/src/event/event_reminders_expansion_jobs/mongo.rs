use crate::shared::repo::DeleteResult;
use crate::{event::IEventRemindersExpansionJobsRepo, shared::mongo_repo};
use mongo_repo::MongoDocument;
use mongodb::{
    bson::doc,
    bson::{oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_core::EventRemindersExpansionJob;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct EventRemindersExpansionsJobRepo {
    collection: Collection,
}

impl EventRemindersExpansionsJobRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-event-reminders-expansion-jobs"),
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersExpansionJobsRepo for EventRemindersExpansionsJobRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> Result<(), Box<dyn Error>> {
        match mongo_repo::bulk_insert::<_, EventRemindersExpansionJobMongo>(&self.collection, jobs)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Ok(()), // fix this
        }
    }

    async fn delete_all_before(&self, before_inc: i64) -> Vec<EventRemindersExpansionJob> {
        let filter = doc! {
            "remind_at": {
                "$lte": before_inc
            }
        };

        // Find before deleting
        let docs = match mongo_repo::find_many_by::<_, EventRemindersExpansionJobMongo>(
            &self.collection,
            filter.clone(),
        )
        .await
        {
            Ok(docs) => docs,
            Err(err) => {
                println!("Error: {:?}", err);
                return vec![];
            }
        };

        // Now delete
        if let Err(err) = self.collection.delete_many(filter, None).await {
            println!("Error: {:?}", err);
        }

        docs
    }

    async fn delete_by_event(&self, event_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let filter = doc! {
            "event_id": event_id
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
struct EventRemindersExpansionJobMongo {
    _id: ObjectId,
    event_id: String,
    dirty: bool,
    timestamp: i64,
}

impl MongoDocument<EventRemindersExpansionJob> for EventRemindersExpansionJobMongo {
    fn to_domain(&self) -> EventRemindersExpansionJob {
        EventRemindersExpansionJob {
            id: self._id.to_string(),
            event_id: self.event_id.clone(),
            timestamp: self.timestamp,
            dirty: self.dirty,
        }
    }

    fn from_domain(job: &EventRemindersExpansionJob) -> Self {
        Self {
            _id: ObjectId::with_string(&job.id).unwrap(),
            event_id: job.event_id.to_owned(),
            timestamp: job.timestamp.to_owned(),
            dirty: job.dirty,
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
