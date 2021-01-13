use std::collections::HashMap;

use crate::{account::domain::Account, event::domain::Reminder};
use crate::{api::Context, event::domain::event::CalendarEvent};
use crate::{calendar::domain::calendar_view::CalendarView, shared::usecase::Usecase};
use chrono::prelude::*;
use mongodb::bson::oid::ObjectId;

/// Creates EventReminders for a calendar event
pub struct SendEventRemindersUseCase {
    event: CalendarEvent,
}

struct SendEventRemindersConfig {
    expansion_interval: i64,
}

impl SendEventRemindersUseCase {
    fn get_config() -> SendEventRemindersConfig {
        SendEventRemindersConfig {
            expansion_interval: 0,
        }
    }
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for SendEventRemindersUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    /// This will run every minute
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        // Find all occurences for the next minute and delete them
        let ts = Utc::now().timestamp_millis() + 60*1000;
        let reminders= ctx.repos.reminder_repo.delete_all_before(ts).await;
        let mut account_lookup = HashMap::new();
        let mut account_reminders: HashMap<String, (&Account, Vec<&Reminder>)> = HashMap::new();

        for reminder in &reminders {
           let account = match account_lookup.get(&reminder.account_id) {
               Some(acc) => acc,
               None => {
                let acc = ctx.repos.account_repo.find(&reminder.account_id).await.unwrap();
                account_lookup.insert(reminder.account_id, acc).as_ref().unwrap()
               }
           };
           match account_reminders.get(&account.id) {
               Some(acc_reminders) => {
                   acc_reminders.1.push(reminder);
               },
               None => {
                   account_reminders.insert(account.id, (account, vec![reminder]));
               }
           };
        }

        for (_, (acc, reminders)) in account_reminders.into_iter()  {

            actix_web::client::Client::new()
                .post(acc.id)
                .send_body(reminders).await;
        }


        // TODOS PREQREQUISITES: ?
        // - create reminders dto that can be serialized
        Ok(())
    }
}
