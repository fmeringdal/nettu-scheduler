use super::IScheduleRepo;
use crate::schedule::domain::Schedule;
use crate::shared::inmemory_repo::*;
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

    async fn delete(&self, schedule_id: &str) -> Option<Schedule> {
        delete(schedule_id, &self.schedules)
    }
}
