use super::ICalendarRepo;
use crate::shared::{inmemory_repo::*, repo::DeleteResult};
use nettu_scheduler_core::Calendar;
use std::error::Error;

pub struct InMemoryCalendarRepo {
    calendars: std::sync::Mutex<Vec<Calendar>>,
}

impl InMemoryCalendarRepo {
    pub fn new() -> Self {
        Self {
            calendars: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for InMemoryCalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()> {
        insert(calendar, &self.calendars);
        Ok(())
    }

    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()> {
        save(calendar, &self.calendars);
        Ok(())
    }

    async fn find(&self, calendar_id: &str) -> Option<Calendar> {
        find(calendar_id, &self.calendars)
    }

    async fn find_by_user(&self, user_id: &str) -> Vec<Calendar> {
        find_by(&self.calendars, |cal| cal.user_id == user_id)
    }

    async fn delete(&self, calendar_id: &str) -> Option<Calendar> {
        delete(calendar_id, &self.calendars)
    }

    async fn delete_by_user(&self, user_id: &str) -> anyhow::Result<DeleteResult> {
        let res = delete_by(&self.calendars, |cal| cal.user_id == user_id);
        Ok(res)
    }
}
