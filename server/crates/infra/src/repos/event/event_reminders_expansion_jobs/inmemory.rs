use super::IEventRemindersExpansionJobsRepo;
use crate::repos::shared::inmemory_repo::*;
use crate::repos::shared::repo::DeleteResult;
use nettu_scheduler_domain::EventRemindersExpansionJob;
use std::error::Error;

pub struct InMemoryEventRemindersExpansionJobsRepo {
    jobs: std::sync::Mutex<Vec<EventRemindersExpansionJob>>,
}

impl InMemoryEventRemindersExpansionJobsRepo {
    pub fn new() -> Self {
        Self {
            jobs: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IEventRemindersExpansionJobsRepo for InMemoryEventRemindersExpansionJobsRepo {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()> {
        for job in jobs {
            insert(job, &self.jobs);
        }
        Ok(())
    }

    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob> {
        // println!("Reminders: {:?}", self.reminders);
        find_and_delete_by(&self.jobs, |reminder| reminder.timestamp <= before)
    }

    async fn delete_by_event(&self, event_id: &str) -> anyhow::Result<DeleteResult> {
        let res = delete_by(&self.jobs, |job| job.event_id == event_id);
        Ok(res)
    }
}
