use crate::shared::usecase::UseCase;
use actix_web::rt::time::Instant;
use nettu_scheduler_domain::{Account, CalendarEvent, Reminder};
use nettu_scheduler_infra::NettuContext;
use std::time::Duration;
use std::{cmp::Ordering, collections::HashMap};
use tracing::error;

/// Creates EventReminders for a calendar event
#[derive(Debug)]
pub struct GetUpcomingRemindersUseCase {
    /// Will fetch reminders for this interval
    pub reminders_interval: i64,
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
        .accounts
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
    reminders.sort_by(|r1, r2| {
        match r2
            .event_id
            .to_string()
            .partial_cmp(&r1.event_id.to_string())
            .unwrap()
        {
            // Highest priority first
            Ordering::Equal => r2.priority.partial_cmp(&r1.priority).unwrap(),
            val => val,
        }
    });

    for i in 1..reminders.len() {
        // Two reminders for the same event_id, remove the one
        // with the lowest priority (e.g. the last one because of the sorting)
        if reminders[i].event_id == reminders[i - 1].event_id {
            reminders.remove(i);
        }
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
            .reminders
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

    const NAME: &'static str = "GetUpcomingReminders";

    /// This will run every minute
    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        // Find all occurences for the next interval and delete them
        let ts = ctx.sys.get_timestamp_millis() + self.reminders_interval;

        // Get all reminders and filter out invalid / expired reminders
        let mut reminders = ctx.repos.reminders.delete_all_before(ts).await;
        dedup_reminders(&mut reminders);
        remove_old_reminders(&mut reminders, ctx).await;

        let event_lookup = ctx
            .repos
            .events
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
            error!("Important: Increase computation time for get reminders usecase");
            Instant::now()
        };
        Ok((grouped_reminders, instant))
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         account,
//         event::{delete_event::DeleteEventUseCase, update_event::UpdateEventUseCase},
//         shared::usecase::execute,
//     };

//     use super::super::create_event::CreateEventUseCase;
//     use super::*;
//     use nettu_scheduler_domain::{Calendar, CalendarEventReminder, User, ID};
//     use nettu_scheduler_infra::{setup_context as _setup_ctx, ISys};
//     use std::sync::Arc;

//     async fn setup_context() -> NettuContext {
//         let ctx = _setup_ctx().await;
//         ctx.repos
//             .reminders
//             .delete_all_before(CalendarEvent::get_max_timestamp())
//             .await;

//         ctx
//     }

//     struct TestContext {
//         ctx: NettuContext,
//         calendar: Calendar,
//         account: Account,
//         user: User,
//     }

//     async fn setup() -> TestContext {
//         let ctx = setup_context().await;
//         let account = Account::default();
//         ctx.repos.accounts.insert(&account).await.unwrap();
//         let user = User::new(account.id.clone());
//         ctx.repos.users.insert(&user).await.unwrap();
//         let calendar = Calendar::new(&user.id, &account.id);
//         ctx.repos.calendars.insert(&calendar).await.unwrap();

//         TestContext {
//             user,
//             account,
//             calendar,
//             ctx,
//         }
//     }

//     fn get_account_id() -> ID {
//         "a574624d-7c7f-456c-bbdd-670710302d45"
//             .parse()
//             .expect("Valid ID")
//     }

//     fn reminder_factory(event_id: &ID, priority: i64) -> Reminder {
//         Reminder {
//             id: Default::default(),
//             account_id: get_account_id(),
//             event_id: event_id.clone(),
//             priority,
//             remind_at: 200,
//         }
//     }

//     #[test]
//     fn should_dedup_reminders() {
//         let mut reminders = Vec::new();
//         dedup_reminders(&mut reminders);
//         assert_eq!(reminders.len(), 0);

//         let event_id = ID::default();
//         let event_id_2 = ID::default();
//         let mut reminders = vec![
//             reminder_factory(&event_id, 0),
//             reminder_factory(&event_id_2, 0),
//         ];
//         dedup_reminders(&mut reminders);
//         assert_eq!(reminders.len(), 2);

//         let mut reminders = vec![
//             reminder_factory(&event_id, 1),
//             reminder_factory(&event_id, 0),
//         ];
//         dedup_reminders(&mut reminders);
//         assert_eq!(reminders.len(), 1);

//         let mut reminders = vec![
//             reminder_factory(&event_id, 0),
//             reminder_factory(&event_id, 1),
//         ];
//         dedup_reminders(&mut reminders);
//         assert_eq!(reminders.len(), 1);
//     }

//     #[actix_web::main]
//     #[serial_test::serial]
//     #[test]
//     async fn removes_old_priorites() {
//         let ctx = setup().await;
//         let availability_event1 = CalendarEvent {
//             id: Default::default(),
//             account_id: ctx.account.id.clone(),
//             busy: false,
//             calendar_id: ctx.calendar.id.clone(),
//             duration: 1000 * 60 * 60,
//             end_ts: 0,
//             exdates: Vec::new(),
//             recurrence: None,
//             start_ts: 1000 * 60 * 60,
//             user_id: ctx.user.id.clone(),
//             reminder: None,
//             is_service: false,
//             metadata: Default::default(),
//             updated: Default::default(),
//             created: Default::default(),
//         };

//         let event_id = ID::default();
//         let reminder_p1 = reminder_factory(&event_id, 1);
//         ctx.ctx
//             .repos
//             .reminders
//             .bulk_insert(&[reminder_p1])
//             .await
//             .unwrap();

//         let reminder_p0 = reminder_factory(&event_id, 0);
//         let mut reminders = vec![reminder_p0];
//         remove_old_reminders(&mut reminders, &ctx.ctx).await;
//         assert_eq!(reminders.len(), 0);

//         // let ctx = setup_context().await;

//         let event_id = ID::default();
//         let reminder_p0 = reminder_factory(&event_id, 0);
//         ctx.repos
//             .reminders
//             .bulk_insert(&[reminder_p0])
//             .await
//             .unwrap();

//         let reminder_p1 = reminder_factory(&event_id, 1);
//         let mut reminders = vec![reminder_p1];
//         remove_old_reminders(&mut reminders, &ctx).await;
//         assert_eq!(reminders.len(), 1);
//     }

//     pub struct StaticTimeSys1;
//     impl ISys for StaticTimeSys1 {
//         fn get_timestamp_millis(&self) -> i64 {
//             1613862000000 // Sun Feb 21 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
//         }
//     }

//     pub struct StaticTimeSys2;
//     impl ISys for StaticTimeSys2 {
//         fn get_timestamp_millis(&self) -> i64 {
//             1613862000000 + 1000 * 60 * 49 // Sun Feb 21 2021 00:49:00 GMT+0100 (Central European Standard Time) {}
//         }
//     }

//     pub struct StaticTimeSys3;
//     impl ISys for StaticTimeSys3 {
//         fn get_timestamp_millis(&self) -> i64 {
//             1613862000000 + 1000 * 60 * 60 * 24 // Sun Feb 22 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
//         }
//     }

//     async fn insert_common_data(ctx: &NettuContext) -> (Account, ID, Calendar) {
//         let account = Account::default();
//         ctx.repos.accounts.insert(&account).await.unwrap();

//         let user = User::new(account.id.clone());
//         ctx.repos.users.insert(&user).await.unwrap();
//         let mut calendar = Calendar::new(&user.id, &account.id);
//         calendar.settings.timezone = chrono_tz::Europe::Oslo;
//         ctx.repos.calendars.insert(&calendar).await.unwrap();
//         (account, user.id, calendar)
//     }

//     async fn insert_events(ctx: &NettuContext) {
//         let (account, user_id, calendar) = insert_common_data(ctx).await;

//         let usecase = CreateEventUseCase {
//             account_id: account.id.clone(),
//             calendar_id: calendar.id.clone(),
//             user_id: user_id.clone(),
//             start_ts: ctx.sys.get_timestamp_millis(),
//             duration: 1000 * 60 * 60 * 2,
//             busy: false,
//             recurrence: Some(Default::default()),
//             reminder: Some(CalendarEventReminder { minutes_before: 10 }),
//             is_service: false,
//             metadata: Default::default(),
//         };

//         execute(usecase, ctx).await.unwrap();

//         let sys3 = StaticTimeSys3 {};
//         let usecase = CreateEventUseCase {
//             account_id: account.id.clone(),
//             calendar_id: calendar.id.clone(),
//             user_id,
//             start_ts: sys3.get_timestamp_millis() + 1000 * 60 * 5,
//             duration: 1000 * 60 * 60 * 2,
//             busy: false,
//             recurrence: None,
//             reminder: Some(CalendarEventReminder { minutes_before: 10 }),
//             is_service: false,
//             metadata: Default::default(),
//         };

//         execute(usecase, ctx).await.unwrap();
//     }

//     #[actix_web::main]
//     #[serial_test::serial]
//     #[test]
//     async fn get_upcoming_reminders() {
//         let mut ctx = setup_context().await;
//         ctx.sys = Arc::new(StaticTimeSys1 {});

//         insert_events(&ctx).await;

//         let usecase = GetUpcomingRemindersUseCase {
//             reminders_interval: 1000 * 60,
//         };
//         let res = execute(usecase, &ctx).await;
//         assert!(res.is_ok());
//         let res = res.unwrap().0;
//         assert_eq!(res.len(), 1);
//         assert_eq!(res[0].1.events.len(), 1);

//         ctx.sys = Arc::new(StaticTimeSys2 {});
//         let usecase = GetUpcomingRemindersUseCase {
//             reminders_interval: 1000 * 60,
//         };
//         let res = execute(usecase, &ctx).await;
//         assert!(res.is_ok());
//         let res = res.unwrap().0;
//         assert_eq!(res.len(), 0);

//         ctx.sys = Arc::new(StaticTimeSys3 {});
//         let usecase = GetUpcomingRemindersUseCase {
//             reminders_interval: 1000 * 60,
//         };
//         let res = execute(usecase, &ctx).await;
//         assert!(res.is_ok());
//         let res = res.unwrap().0;
//         assert_eq!(res.len(), 1);
//         assert_eq!(res[0].1.events.len(), 2);

//         let usecase = GetUpcomingRemindersUseCase {
//             reminders_interval: 1000 * 60,
//         };
//         let res = execute(usecase, &ctx).await;
//         assert!(res.is_ok());
//         let res = res.unwrap().0;
//         assert_eq!(res.len(), 0);
//     }

//     #[actix_web::main]
//     #[serial_test::serial]
//     #[test]
//     async fn updating_event_also_updates_reminders() {
//         let mut ctx = setup_context().await;
//         ctx.sys = Arc::new(StaticTimeSys1 {});

//         let now = ctx.sys.get_timestamp_millis();
//         let minutes_before = 10;

//         let (account, user_id, calendar) = insert_common_data(&ctx).await;
//         let usecase = CreateEventUseCase {
//             account_id: account.id.clone(),
//             calendar_id: calendar.id.clone(),
//             user_id: user_id.clone(),
//             start_ts: now,
//             duration: 1000 * 60 * 60 * 2,
//             busy: false,
//             recurrence: Some(Default::default()),
//             reminder: Some(CalendarEventReminder { minutes_before }),
//             is_service: false,
//             metadata: Default::default(),
//         };

//         let calendar_event = execute(usecase, &ctx).await.unwrap();
//         let old_reminders = ctx.repos.reminders.delete_all_before(now).await;
//         ctx.repos
//             .reminders
//             .bulk_insert(&old_reminders)
//             .await
//             .unwrap();

//         let start_ts_diff = 15 * 60 * 1000; // 15 minutes
//         let new_start = calendar_event.start_ts + start_ts_diff; // Postponed 15 minutes
//         let update_event_usecase = UpdateEventUseCase {
//             event_id: calendar_event.id,
//             busy: None,
//             duration: None,
//             exdates: None,
//             metadata: None,
//             reminder: Some(CalendarEventReminder { minutes_before }),
//             recurrence: Some(Default::default()),
//             is_service: None,
//             start_ts: Some(new_start),
//             user_id: calendar_event.user_id,
//         };
//         execute(update_event_usecase, &ctx).await.unwrap();
//         let new_reminders = ctx.repos.reminders.delete_all_before(new_start).await;
//         assert_eq!(new_reminders.len(), old_reminders.len());
//         assert_eq!(new_reminders.len(), 1);
//         assert_eq!(
//             new_reminders[0].remind_at + minutes_before * 60 * 1000,
//             new_start
//         );
//         assert_eq!(new_reminders[0].event_id, old_reminders[0].event_id);
//         assert_eq!(
//             new_reminders[0].remind_at,
//             old_reminders[0].remind_at + start_ts_diff
//         );
//     }

//     #[actix_web::main]
//     #[serial_test::serial]
//     #[test]
//     async fn deleting_event_reminder_setting_also_deletes_reminders() {
//         let mut ctx = setup_context().await;
//         ctx.sys = Arc::new(StaticTimeSys1 {});

//         let now = ctx.sys.get_timestamp_millis();

//         let (account, user_id, calendar) = insert_common_data(&ctx).await;
//         let usecase = CreateEventUseCase {
//             account_id: account.id.clone(),
//             calendar_id: calendar.id.clone(),
//             user_id: user_id.clone(),
//             start_ts: now,
//             duration: 1000 * 60 * 60 * 2,
//             busy: false,
//             recurrence: Some(Default::default()),
//             reminder: Some(CalendarEventReminder { minutes_before: 10 }),
//             is_service: false,
//             metadata: Default::default(),
//         };

//         let calendar_event = execute(usecase, &ctx).await.unwrap();
//         let old_reminders = ctx.repos.reminders.delete_all_before(now).await;
//         ctx.repos
//             .reminders
//             .bulk_insert(&old_reminders)
//             .await
//             .unwrap();

//         let update_event_usecase = UpdateEventUseCase {
//             event_id: calendar_event.id,
//             busy: None,
//             duration: None,
//             exdates: None,
//             metadata: None,
//             reminder: None,
//             recurrence: Some(Default::default()),
//             is_service: None,
//             start_ts: None,
//             user_id: calendar_event.user_id,
//         };
//         execute(update_event_usecase, &ctx).await.unwrap();
//         let new_reminders = ctx.repos.reminders.delete_all_before(now).await;
//         assert!(new_reminders.is_empty());
//     }

//     #[actix_web::main]
//     #[serial_test::serial]
//     #[test]
//     async fn deleting_event_also_deletes_reminders() {
//         let mut ctx = setup_context().await;
//         ctx.sys = Arc::new(StaticTimeSys1 {});

//         let now = ctx.sys.get_timestamp_millis();

//         let (account, user_id, calendar) = insert_common_data(&ctx).await;
//         let usecase = CreateEventUseCase {
//             account_id: account.id.clone(),
//             calendar_id: calendar.id.clone(),
//             user_id: user_id.clone(),
//             start_ts: now,
//             duration: 1000 * 60 * 60 * 2,
//             busy: false,
//             recurrence: Some(Default::default()),
//             reminder: Some(CalendarEventReminder { minutes_before: 10 }),
//             is_service: false,
//             metadata: Default::default(),
//         };

//         let calendar_event = execute(usecase, &ctx).await.unwrap();
//         let old_reminders = ctx.repos.reminders.delete_all_before(now).await;
//         ctx.repos
//             .reminders
//             .bulk_insert(&old_reminders)
//             .await
//             .unwrap();

//         let update_event_usecase = DeleteEventUseCase {
//             event_id: calendar_event.id,
//             user_id: calendar_event.user_id,
//         };
//         execute(update_event_usecase, &ctx).await.unwrap();
//         let new_reminders = ctx.repos.reminders.delete_all_before(now).await;
//         assert!(new_reminders.is_empty());
//     }
// }
