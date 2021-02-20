use crate::event::{
    domain::Reminder,
    repos::{DeleteResult, IReminderRepo},
};
use crate::shared::inmemory_repo::*;
use std::error::Error;

pub struct InMemoryReminderRepo {
    reminders: std::sync::Mutex<Vec<Reminder>>,
}

impl InMemoryReminderRepo {
    pub fn new() -> Self {
        Self {
            reminders: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl IReminderRepo for InMemoryReminderRepo {
    async fn bulk_insert(
        &self,
        reminders: &[crate::event::domain::Reminder],
    ) -> Result<(), Box<dyn Error>> {
        for reminder in reminders {
            insert(reminder, &self.reminders);
        }
        Ok(())
    }

    async fn delete_all_before(&self, before: i64) -> Vec<Reminder> {
        find_and_delete_by(&self.reminders, |reminder| reminder.remind_at <= before)
    }

    async fn delete_by_event(&self, event_id: &str) -> Result<DeleteResult, Box<dyn Error>> {
        let res = delete_by(&self.reminders, |reminder| reminder.event_id == event_id);
        Ok(res)
    }
}
