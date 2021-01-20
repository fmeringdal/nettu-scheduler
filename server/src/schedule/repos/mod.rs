mod inmemory;
mod mongo;

pub use inmemory::InMemoryScheduleRepo;
pub use mongo::ScheduleRepo;

use std::error::Error;

use super::domain::Schedule;

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>>;
    async fn save(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>>;
    async fn find(&self, schedule_id: &str) -> Option<Schedule>;
    async fn find_many(&self, schedule_ids: &[String]) -> Vec<Schedule>;
    async fn find_by_user(&self, user_id: &str) -> Vec<Schedule>;
    async fn delete(&self, schedule_id: &str) -> Option<Schedule>;
}
