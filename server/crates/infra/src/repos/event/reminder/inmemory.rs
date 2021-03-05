use super::IReminderRepo;
use crate::repos::shared::inmemory_repo::*;
use crate::repos::shared::repo::DeleteResult;
use nettu_scheduler_domain::{Reminder, ID};

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
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()> {
        for reminder in reminders {
            insert(reminder, &self.reminders);
        }
        Ok(())
    }

    async fn find_by_event_and_priority(&self, event_id: &ID, priority: i64) -> Option<Reminder> {
        let reminders = find_by(&self.reminders, |reminder| {
            reminder.event_id == *event_id && reminder.priority == priority
        });
        if reminders.is_empty() {
            return None;
        }
        return Some(reminders[0].clone());
    }

    async fn delete_all_before(&self, before: i64) -> Vec<Reminder> {
        find_and_delete_by(&self.reminders, |reminder| reminder.remind_at <= before)
    }

    async fn delete_by_events(&self, event_ids: &[ID]) -> anyhow::Result<DeleteResult> {
        let res = delete_by(&self.reminders, |reminder| {
            event_ids.contains(&reminder.event_id)
        });
        Ok(res)
    }
}
