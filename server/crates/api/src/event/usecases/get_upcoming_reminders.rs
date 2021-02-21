use crate::shared::usecase::UseCase;
use nettu_scheduler_core::{Account, CalendarEvent, Reminder};
use nettu_scheduler_infra::Context;
use std::collections::HashMap;

/// Creates EventReminders for a calendar event
pub struct GetUpcomingRemindersUseCase {}

struct SendEventRemindersConfig {
    send_interval: i64,
}

impl GetUpcomingRemindersUseCase {
    fn get_config() -> SendEventRemindersConfig {
        SendEventRemindersConfig {
            send_interval: 60 * 1000, // every minute
        }
    }
}

#[derive(Debug)]
pub enum UseCaseErrors {}

#[derive(Debug)]
pub struct AccountEventReminders {
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
) -> Vec<(Account, AccountEventReminders)> {
    let account_lookup = get_accounts_from_reminders(&reminders, ctx).await;

    let mut account_reminders: HashMap<String, (&Account, Vec<CalendarEvent>)> = HashMap::new();

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
                account_reminders.insert(account.id.to_owned(), (account, vec![calendar_event]));
            }
        };
    }

    account_reminders
        .into_iter()
        .map(|(_, (acc, events))| (acc.clone(), AccountEventReminders { events }))
        .collect()
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUpcomingRemindersUseCase {
    type Response = Vec<(Account, AccountEventReminders)>;

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

        let grouped_reminders = create_reminders_for_accounts(reminders, event_lookup, ctx).await;

        // let client = actix_web::client::Client::new();
        // for (acc, reminders) in account_reminders {
        //     match acc.settings.webhook {
        //         None => continue,
        //         Some(webhook) => {
        //             if let Err(e) = client
        //                 .post(webhook.url)
        //                 .header("nettu-scheduler-webhook-key", webhook.key)
        //                 .send_json(&reminders)
        //                 .await
        //             {
        //                 println!("Error informing client of reminders: {:?}", e);
        //             }
        //         }
        //     }
        // }

        Ok(grouped_reminders)
    }
}
