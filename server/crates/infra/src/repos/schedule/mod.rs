mod inmemory;
mod mongo;

use crate::repos::shared::repo::DeleteResult;
pub use inmemory::InMemoryScheduleRepo;
pub use mongo::MongoScheduleRepo;
use nettu_scheduler_domain::Schedule;
use std::error::Error;

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn find(&self, schedule_id: &str) -> Option<Schedule>;
    async fn find_many(&self, schedule_ids: &[String]) -> Vec<Schedule>;
    async fn find_by_user(&self, user_id: &str) -> Vec<Schedule>;
    async fn delete(&self, schedule_id: &str) -> Option<Schedule>;
    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult>;
}
