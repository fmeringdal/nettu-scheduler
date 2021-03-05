use crate::shared::usecase::UseCase;
use actix_web::rt::time::Instant;
use nettu_scheduler_domain::{Account, CalendarEvent, Reminder};
use nettu_scheduler_infra::NettuContext;
use std::collections::HashMap;
use std::time::Duration;

/// Creates EventReminders for a calendar event
#[derive(Debug)]
pub struct GetUpcomingRemindersUseCase {}

struct SendEventRemindersConfig {
    send_interval: i64,
    computation_time: i64,
}

impl GetUpcomingRemindersUseCase {
    fn get_config() -> SendEventRemindersConfig {
        SendEventRemindersConfig {
            send_interval: 60 * 1000,        // every minute
            computation_time: 3 * 60 * 1000, // Expect to be able to get all reminders in 3 minutes
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
        .map(|a| (a.id.as_string(), a))
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
        let account = match account_lookup.get(&reminder.account_id.as_string()) {
            Some(a) => a,
            None => continue,
        };

        // Remove instead of get because there shouldnt be multiple reminders for the same event id
        // and also we get ownership over calendar_event
        let calendar_event = match event_lookup.remove(&reminder.event_id.as_string()) {
            Some(e) => e,
            None => continue,
        };
        match account_reminders.get_mut(&account.id.as_string()) {
            Some(acc_reminders) => {
                acc_reminders.1.push(calendar_event);
            }
            None => {
                account_reminders.insert(account.id.as_string(), (account, vec![calendar_event]));
            }
        };
    }

    account_reminders
        .into_iter()
        .map(|(_, (acc, events))| (acc.clone(), AccountEventReminders { events }))
        .collect()
}

// Remove possible duplicate reminders created by the two triggers
// of sync event reminders
fn dedup_reminders(reminders: &mut Vec<Reminder>) {
    reminders.sort_by_key(|r1| r1.priority);
    let mut reminders_count = reminders.len();
    let mut index = 0;
    while index < reminders_count {
        for j in index + 1..reminders_count {
            if reminders[index].event_id == reminders[j].event_id {
                reminders.remove(j);
                reminders_count -= 1;
                // There will always just be maximum two reminders duplicated
                // so it is okay to break once the duplicate is found
                break;
            }
        }

        index += 1;
    }
}

// Detects if there have been generated reminders with higher priority in the reminder repo and deletes
// the old one if that is the case
async fn remove_old_reminders(reminders: &mut Vec<Reminder>, ctx: &NettuContext) {
    // Priority 0 reminders
    let reminders_p0 = reminders
        .iter()
        .filter(|r| r.priority == 0)
        .collect::<Vec<_>>();

    let mut event_ids_to_remove = HashMap::new();

    for i in (0..reminders_p0.len()).rev() {
        let reminder = reminders_p0[i];
        if ctx
            .repos
            .reminder_repo
            .find_by_event_and_priority(&reminder.event_id, 1)
            .await
            .is_some()
        {
            event_ids_to_remove.insert(reminder.event_id.as_string(), ());
        }
    }

    reminders.retain(|r| !event_ids_to_remove.contains_key(&r.event_id.as_string()));
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUpcomingRemindersUseCase {
    type Response = (Vec<(Account, AccountEventReminders)>, Instant);

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    /// This will run every minute
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        // Find all occurences for the next interval and delete them
        let conf = Self::get_config();
        let ts = ctx.sys.get_timestamp_millis() + conf.send_interval + conf.computation_time;

        // Get all reminders and filter out invalid / expired reminders
        let mut reminders = ctx.repos.reminder_repo.delete_all_before(ts).await;
        dedup_reminders(&mut reminders);
        remove_old_reminders(&mut reminders, ctx).await;

        let event_lookup = ctx
            .repos
            .event_repo
            .find_many(
                &reminders
                    .iter()
                    .map(|r| r.event_id.clone())
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap()
            .into_iter()
            .map(|e| (e.id.as_string(), e))
            .collect::<HashMap<_, _>>();

        let grouped_reminders = create_reminders_for_accounts(reminders, event_lookup, ctx).await;

        let millis_to_send = ts - ctx.sys.get_timestamp_millis();
        let instant = if millis_to_send > 0 {
            Instant::now() + Duration::from_millis(millis_to_send as u64)
        } else {
            println!("Important: Increase computation time for get reminders usecase");
            Instant::now()
        };
        Ok((grouped_reminders, instant))
    }
}

#[cfg(test)]
mod tests {
    use super::super::create_event::CreateEventUseCase;
    use super::*;
    use nettu_scheduler_domain::{
        Calendar, CalendarEventReminder, RRuleFrequenzy, RRuleOptions, ID,
    };
    use nettu_scheduler_infra::{setup_context, ISys};
    use std::sync::Arc;

    fn get_account_id() -> ID {
        "507f1f77bcf86cd799439011".parse().expect("Valid ID")
    }

    fn reminder_factory(event_id: &ID, priority: i64) -> Reminder {
        Reminder {
            id: Default::default(),
            account_id: get_account_id(),
            event_id: event_id.clone(),
            priority,
            remind_at: 200,
        }
    }

    #[test]
    fn should_dedup_reminders() {
        let mut reminders = vec![];
        dedup_reminders(&mut reminders);
        assert_eq!(reminders.len(), 0);

        let event_id = ID::default();
        let event_id_2 = ID::default();
        let mut reminders = vec![
            reminder_factory(&event_id, 0),
            reminder_factory(&event_id_2, 0),
        ];
        dedup_reminders(&mut reminders);
        assert_eq!(reminders.len(), 2);

        let mut reminders = vec![
            reminder_factory(&event_id, 1),
            reminder_factory(&event_id, 0),
        ];
        dedup_reminders(&mut reminders);
        assert_eq!(reminders.len(), 1);

        let mut reminders = vec![
            reminder_factory(&event_id, 0),
            reminder_factory(&event_id, 1),
        ];
        dedup_reminders(&mut reminders);
        assert_eq!(reminders.len(), 1);
    }

    #[actix_web::main]
    #[test]
    async fn removes_old_priorites() {
        let ctx = setup_context().await;

        let event_id = ID::default();
        let reminder_p1 = reminder_factory(&event_id, 1);
        ctx.repos
            .reminder_repo
            .bulk_insert(&[reminder_p1])
            .await
            .unwrap();

        let reminder_p0 = reminder_factory(&event_id, 0);
        let mut reminders = vec![reminder_p0];
        remove_old_reminders(&mut reminders, &ctx).await;
        assert_eq!(reminders.len(), 0);

        let ctx = setup_context().await;

        let event_id = ID::default();
        let reminder_p0 = reminder_factory(&event_id, 0);
        ctx.repos
            .reminder_repo
            .bulk_insert(&[reminder_p0])
            .await
            .unwrap();

        let reminder_p1 = reminder_factory(&event_id, 1);
        let mut reminders = vec![reminder_p1];
        remove_old_reminders(&mut reminders, &ctx).await;
        assert_eq!(reminders.len(), 1);
    }

    pub struct StaticTimeSys1 {}
    impl ISys for StaticTimeSys1 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 // Sun Feb 21 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    pub struct StaticTimeSys2 {}
    impl ISys for StaticTimeSys2 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 + 1000 * 60 * 49 // Sun Feb 21 2021 00:49:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    pub struct StaticTimeSys3 {}
    impl ISys for StaticTimeSys3 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 + 1000 * 60 * 60 * 24 // Sun Feb 22 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
    }

    async fn insert_events(ctx: &NettuContext) {
        let account = Account::default();
        ctx.repos.account_repo.insert(&account).await.unwrap();

        let user_id = ID::default();
        let mut calendar = Calendar::new(&user_id);
        calendar.settings.timezone = chrono_tz::Europe::Oslo;
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = CreateEventUseCase {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            user_id: user_id.clone(),
            start_ts: ctx.sys.get_timestamp_millis(),
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
            services: vec![],
        };

        usecase.execute(ctx).await.unwrap();

        let sys3 = StaticTimeSys3 {};
        let mut usecase = CreateEventUseCase {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            user_id,
            start_ts: sys3.get_timestamp_millis() + 1000 * 60 * 5,
            duration: 1000 * 60 * 60 * 2,
            busy: false,
            rrule_options: None,
            reminder: Some(CalendarEventReminder { minutes_before: 10 }),
            services: vec![],
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
        let res = res.unwrap().0;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.events.len(), 1);

        ctx.sys = Arc::new(StaticTimeSys2 {});
        let mut usecase = GetUpcomingRemindersUseCase {};
        let res = usecase.execute(&ctx).await;
        println!("2. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 0);

        ctx.sys = Arc::new(StaticTimeSys3 {});
        let mut usecase = GetUpcomingRemindersUseCase {};
        let res = usecase.execute(&ctx).await;
        println!("3. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.events.len(), 2);

        let res = usecase.execute(&ctx).await;
        println!("4. Reminders got: {:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 0);
    }
}
