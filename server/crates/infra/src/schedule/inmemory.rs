use super::IScheduleRepo;
use crate::shared::{inmemory_repo::*, repo::DeleteResult};
use nettu_scheduler_core::Schedule;
use std::error::Error;

pub struct InMemoryScheduleRepo {
    schedules: std::sync::Mutex<Vec<Schedule>>,
}

impl InMemoryScheduleRepo {
    pub fn new() -> Self {
        Self {
            schedules: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for InMemoryScheduleRepo {
    async fn insert(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>> {
        insert(schedule, &self.schedules);
        Ok(())
    }

    async fn save(&self, schedule: &Schedule) -> Result<(), Box<dyn Error>> {
        save(schedule, &self.schedules);
        Ok(())
    }

    async fn find(&self, schedule_id: &str) -> Option<Schedule> {
        find(schedule_id, &self.schedules)
    }

    async fn find_by_user(&self, user_id: &str) -> Vec<Schedule> {
        find_by(&self.schedules, |schedule| schedule.user_id == user_id)
    }

    async fn find_many(&self, schedule_ids: &[String]) -> Vec<Schedule> {
        find_by(&self.schedules, |schedule| {
            schedule_ids.contains(&schedule.id)
        })
    }

    async fn delete(&self, schedule_id: &str) -> Option<Schedule> {
        delete(schedule_id, &self.schedules)
    }

    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult> {
        let res = delete_by(&self.schedules, |schedule| schedule.user_id == user_id);
        Ok(res)
    }
}
