use super::IEventRemindersExpansionJobsRepo;
use crate::repos::shared::mongo_repo;
use crate::repos::shared::repo::DeleteResult;
use mongo_repo::MongoDocument;
use mongodb::{
    bson::doc,
    bson::{oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{EventRemindersExpansionJob, ID};
use serde::{Deserialize, Serialize};

pub struct MongoEventRemindersExpansionsJobRepo {
    collection: Collection,
}

impl MongoEventRemindersExpansionsJobRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("calendar-event-reminders-expansion-jobs"),
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersExpansionJobsRepo for MongoEventRemindersExpansionsJobRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()> {
        mongo_repo::bulk_insert::<_, EventRemindersExpansionJobMongo>(&self.collection, jobs).await
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

    async fn delete_by_event(&self, event_id: &ID) -> anyhow::Result<DeleteResult> {
        let filter = doc! {
            "event_id": event_id.inner_ref()
        };
        self.collection
            .delete_many(filter, None)
            .await
            .map(|res| DeleteResult {
                deleted_count: res.deleted_count,
            })
            .map_err(anyhow::Error::new)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EventRemindersExpansionJobMongo {
    _id: ObjectId,
    event_id: ObjectId,
    timestamp: i64,
}

impl MongoDocument<EventRemindersExpansionJob> for EventRemindersExpansionJobMongo {
    fn to_domain(&self) -> EventRemindersExpansionJob {
        EventRemindersExpansionJob {
            id: ID::from(self._id.clone()),
            event_id: ID::from(self.event_id.clone()),
            timestamp: self.timestamp,
        }
    }

    fn from_domain(job: &EventRemindersExpansionJob) -> Self {
        Self {
            _id: job.id.inner_ref().clone(),
            event_id: job.event_id.inner_ref().clone(),
            timestamp: job.timestamp.to_owned(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": self._id.clone()
        }
    }
}
