mod inmemory;
mod mongo;

use crate::shared::repo::DeleteResult;
pub use inmemory::InMemoryScheduleRepo;
pub use mongo::ScheduleRepo;
use nettu_scheduler_core::Schedule;
use std::error::Error;

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>>;
    async fn save(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>>;
    async fn find(&self, schedule_id: &str) -> Option<Schedule>;
    async fn find_many(&self, schedule_ids: &[String]) -> Vec<Schedule>;
    async fn find_by_user(&self, user_id: &str) -> Vec<Schedule>;
    async fn delete(&self, schedule_id: &str) -> Option<Schedule>;
    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult>;
}
