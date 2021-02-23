use crate::shared::usecase::UseCase;
use nettu_scheduler_core::{Account, CalendarEvent, Reminder};
use nettu_scheduler_infra::NettuContext;
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
    pub events: Vec<CalendarEvent>,
}

async fn get_accounts_from_reminders(
    reminders: &[Reminder],
    ctx: &NettuContext,
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
    ctx: &NettuContext,
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

    type Context = NettuContext;

    /// This will run every minute
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        // Find all occurences for the next interval and delete them
        let ts = ctx.sys.get_utc_timestamp() + Self::get_config().send_interval;
        println!("before ts: {}", ts);
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

        Ok(grouped_reminders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::usecases::create_event::CreateEventUseCase;
    use nettu_scheduler_core::{Calendar, CalendarEventReminder, RRuleFrequenzy, RRuleOptions};
    use nettu_scheduler_infra::{setup_context, ISys};
    use std::sync::Arc;

    pub struct StaticTimeSys1 {}
    impl ISys for StaticTimeSys1 {
        fn get_utc_timestamp(&self) -> i64 {
            1613862000000 // Sun Feb 21 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    pub struct StaticTimeSys2 {}
    impl ISys for StaticTimeSys2 {
        fn get_utc_timestamp(&self) -> i64 {
            1613862000000 + 1000 * 60 * 49 // Sun Feb 21 2021 00:49:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    pub struct StaticTimeSys3 {}
    impl ISys for StaticTimeSys3 {
        fn get_utc_timestamp(&self) -> i64 {
            1613862000000 + 1000 * 60 * 60 * 24 // Sun Feb 22 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    async fn insert_events(ctx: &NettuContext) {
        let account = Account::default();
        ctx.repos.account_repo.insert(&account).await.unwrap();

        let user_id = "1";
        let mut calendar = Calendar::new(user_id.into());
        calendar.settings.timezone = chrono_tz::Europe::Oslo;
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = CreateEventUseCase {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            user_id: user_id.into(),
            start_ts: ctx.sys.get_utc_timestamp(),
            duration: 1000 * 60 * 60 * 2,
            busy: false,
            rrule_options: Some(RRuleOptions {
                freq: RRuleFrequenzy::Daily,
                interval: 1,
                count: None,
                until: None,
                bysetpos: None,
                byweekday: None,
                bynweekday: None,
            }),
            reminder: Some(CalendarEventReminder { minutes_before: 10 }),
        };

        usecase.execute(ctx).await.unwrap();

        let sys3 = StaticTimeSys3 {};
        let mut usecase = CreateEventUseCase {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            user_id: user_id.into(),
            start_ts: sys3.get_utc_timestamp() + 1000 * 60 * 5,
            duration: 1000 * 60 * 60 * 2,
            busy: false,
            rrule_options: None,
            reminder: Some(CalendarEventReminder { minutes_before: 10 }),
        };

        usecase.execute(ctx).await.unwrap();
    }

    #[actix_web::main]
    #[test]
    async fn get_upcoming_reminders() {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(StaticTimeSys1 {});

        insert_events(&ctx).await;

        let mut usecase = GetUpcomingRemindersUseCase {};
        let res = usecase.execute(&ctx).await;
        println!("1. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.events.len(), 1);

        ctx.sys = Arc::new(StaticTimeSys2 {});
        let mut usecase = GetUpcomingRemindersUseCase {};
        let res = usecase.execute(&ctx).await;
        println!("2. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 0);

        ctx.sys = Arc::new(StaticTimeSys3 {});
        let mut usecase = GetUpcomingRemindersUseCase {};
        let res = usecase.execute(&ctx).await;
        println!("3. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.events.len(), 2);

        let res = usecase.execute(&ctx).await;
        println!("4. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.len(), 0);
    }
}
