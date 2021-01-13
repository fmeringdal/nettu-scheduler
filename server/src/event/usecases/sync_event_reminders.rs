use crate::event::domain::Reminder;
use crate::{api::Context, event::domain::event::CalendarEvent};
use crate::{calendar::domain::calendar_view::CalendarView, shared::usecase::Usecase};
use chrono::prelude::*;
use mongodb::bson::oid::ObjectId;

/// Creates EventReminders for a calendar event
pub struct SyncEventRemindersUseCase {
    event: CalendarEvent,
}

struct SyncEventRemindersConfig {
    expansion_interval: i64,
}

impl SyncEventRemindersUseCase {
    fn get_config() -> SyncEventRemindersConfig {
        SyncEventRemindersConfig {
            expansion_interval: 0,
        }
    }
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for SyncEventRemindersUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        // delete existing reminders
        if ctx.repos.reminder_repo.delete_by_event(&self.event.id).await.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        let conf = Self::get_config();
        if self.event.reminder.is_none() {
            return Ok(());
        }
        let millis_before = self.event.reminder.clone().unwrap().minutes_before * 60 * 1000;
        let now = Utc::now().timestamp_millis();
        let reminders_end = now + conf.expansion_interval;
        let expansion_view = CalendarView::create(now, reminders_end).unwrap();
        let reminders = self
            .event
            .expand(Some(&expansion_view))
            .iter()
            .map(|dt| dt.start_ts - millis_before)
            .map(|ts| Reminder {
                event_id: self.event.id.to_owned(),
                account_id: self.event.account_id.to_owned(),
                remind_at: ts,
                id: ObjectId::new().to_string(),
            })
            .collect::<Vec<_>>();

        // create reminders for the next `self.expansion_interval`
        if ctx.repos.reminder_repo.bulk_insert(&reminders).await.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        // TODOS PREQREQUISITES: ?
        Ok(())
    }
}
