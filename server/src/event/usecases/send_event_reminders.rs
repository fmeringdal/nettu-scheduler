use std::collections::HashMap;

use crate::shared::usecase::Usecase;
use crate::{account::domain::Account, event::domain::Reminder};
use crate::{api::Context, event::domain::event::CalendarEvent};
use chrono::prelude::*;
use serde::Serialize;

/// Creates EventReminders for a calendar event
pub struct SendEventRemindersUseCase {}

struct SendEventRemindersConfig {
    send_interval: i64,
}

impl SendEventRemindersUseCase {
    fn get_config() -> SendEventRemindersConfig {
        SendEventRemindersConfig {
            send_interval: 60 * 1000, // every minute
        }
    }
}

#[derive(Debug)]
pub enum UseCaseErrors {
    StorageError,
}

#[derive(Debug, Serialize)]
struct AccountEventReminders {
    events: Vec<CalendarEvent>,
}

async fn get_accounts_from_reminders(
    reminders: &[Reminder],
    ctx: &Context,
) -> HashMap<String, Account> {
    let account_ids: Vec<_> = reminders
        .iter()
        .map(|r| r.account_id.to_owned())
        .collect::<Vec<_>>();
    ctx.repos
        .account_repo
        .find_many(&account_ids)
        .await
        .unwrap()
        .into_iter()
        .map(|a| (a.id.to_owned(), a))
        .collect()
}

async fn create_reminders_for_accounts(
    reminders: Vec<Reminder>,
    mut event_lookup: HashMap<String, CalendarEvent>,
    ctx: &Context,
) -> Vec<(Account, AccountEventReminders )> {
    let account_lookup = get_accounts_from_reminders(&reminders, ctx).await;

    let mut account_reminders: HashMap<String, (&Account, Vec<CalendarEvent>)> =
            HashMap::new();

    for reminder in reminders {
        let account = match account_lookup.get(&reminder.account_id) {
            Some(a) => a,
            None => continue,
        };

        // Remove instead of get because there shouldnt be multiple reminders for the same event id
        // and also we get ownership over calendar_event
        let calendar_event = match event_lookup.remove(&reminder.event_id) {
            Some(e) => e,
            None => continue,
        };
        match account_reminders.get_mut(&account.id) {
            Some(acc_reminders) => {
                acc_reminders.1.push(calendar_event);
            }
            None => {
                account_reminders
                    .insert(account.id.to_owned(), (account, vec![calendar_event]));
            }
        };
    }

    account_reminders
        .into_iter()
        .map(|(_, (acc, events))| (acc.clone(), AccountEventReminders { events }))
        .collect()
}

#[async_trait::async_trait(?Send)]
impl Usecase for SendEventRemindersUseCase {
    type Response = ();

    type Errors = UseCaseErrors;

    type Context = Context;

    /// This will run every minute
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        // Find all occurences for the next interval and delete them
        let ts = ctx.sys.get_utc_timestamp() + Self::get_config().send_interval;
        let reminders = ctx.repos.reminder_repo.delete_all_before(ts).await;
        let event_lookup = ctx
            .repos
            .event_repo
            .find_many(
                &reminders
                    .iter()
                    .map(|r| r.event_id.to_owned())
                    .collect::<Vec<String>>(),
            )
            .await
            .unwrap()
            .into_iter()
            .map(|e| (e.id.to_owned(), e))
            .collect::<HashMap<_, _>>();

        let account_reminders = create_reminders_for_accounts(reminders, event_lookup, ctx).await;
        
        let client = actix_web::client::Client::new();
        for (acc, reminders) in account_reminders.into_iter().filter(|(acc, _)| acc.settings.webhook_url.is_some()) {
                if let Err(e) = client
                    .post(acc.settings.webhook_url.unwrap())
                    .header(
                        "nettu-scheduler-webhook-key",
                        acc.settings.webhook_key.to_owned().unwrap(),
                    )
                    .send_json(&reminders)
                    .await
                {
                    println!("Error informing client of reminders: {:?}", e);
                }
        }

        Ok(())
    }
}
