mod inmemory;
mod mongo;

use crate::repos::shared::repo::DeleteResult;
pub use inmemory::InMemoryScheduleRepo;
pub use mongo::MongoScheduleRepo;
use nettu_scheduler_domain::{Schedule, ID};

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn find(&self, schedule_id: &ID) -> Option<Schedule>;
    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule>;
    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule>;
    async fn delete(&self, schedule_id: &ID) -> Option<Schedule>;
    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult>;
}
