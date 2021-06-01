use super::IScheduleRepo;
use crate::{
    repos::shared::{inmemory_repo::*, repo::DeleteResult},
    MetadataFindQuery,
};
use nettu_scheduler_domain::{Schedule, ID};

pub struct InMemoryScheduleRepo {
    schedules: std::sync::Mutex<Vec<Schedule>>,
}

impl InMemoryScheduleRepo {
    pub fn new() -> Self {
        Self {
            schedules: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for InMemoryScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()> {
        insert(schedule, &self.schedules);
        Ok(())
    }

    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()> {
        save(schedule, &self.schedules);
        Ok(())
    }

    async fn find(&self, schedule_id: &ID) -> Option<Schedule> {
        find(schedule_id, &self.schedules)
    }

    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule> {
        find_by(&self.schedules, |schedule| {
            schedule_ids.contains(&schedule.id)
        })
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule> {
        find_by(&self.schedules, |schedule| schedule.user_id == *user_id)
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Schedule> {
        find_by_metadata(&self.schedules, query)
    }

    async fn delete(&self, schedule_id: &ID) -> Option<Schedule> {
        delete(schedule_id, &self.schedules)
    }

    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult> {
        let res = delete_by(&self.schedules, |schedule| schedule.user_id == *user_id);
        Ok(res)
    }
}
