mod postgres;

use nettu_scheduler_domain::EventRemindersExpansionJob;
pub use postgres::PostgresEventReminderGenerationJobsRepo;

#[async_trait::async_trait]
pub trait IEventRemindersGenerationJobsRepo: Send + Sync {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()>;
    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, Calendar, CalendarEvent, EventRemindersExpansionJob, User,
    };

    #[tokio::test]
    async fn crud() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        let e1 = CalendarEvent {
            id: Default::default(),
            account_id: account.id.clone(),
            busy: false,
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: user.id.clone(),
            reminders: Vec::new(),
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        ctx.repos.events.insert(&e1).await.unwrap();
        let e2 = CalendarEvent {
            id: Default::default(),
            account_id: account.id.clone(),
            busy: false,
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: user.id.clone(),
            reminders: Vec::new(),
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        ctx.repos.events.insert(&e2).await.unwrap();
        let e3 = CalendarEvent {
            id: Default::default(),
            account_id: account.id.clone(),
            busy: false,
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: user.id.clone(),
            reminders: Vec::new(),
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        ctx.repos.events.insert(&e3).await.unwrap();

        let v_e1 = ctx
            .repos
            .reminders
            .init_version(&e1.id)
            .await
            .expect("To create reminder version");
        let v_e2 = ctx
            .repos
            .reminders
            .init_version(&e2.id)
            .await
            .expect("To create reminder version");
        let v_e3 = ctx
            .repos
            .reminders
            .init_version(&e3.id)
            .await
            .expect("To create reminder version");

        let jobs = vec![
            EventRemindersExpansionJob {
                event_id: e1.id.clone(),
                timestamp: 1,
                version: v_e1,
            },
            EventRemindersExpansionJob {
                event_id: e2.id.clone(),
                timestamp: 2,
                version: v_e2,
            },
            EventRemindersExpansionJob {
                event_id: e3.id.clone(),
                timestamp: 3,
                version: v_e3,
            },
        ];
        assert!(ctx
            .repos
            .event_reminders_generation_jobs
            .bulk_insert(&jobs)
            .await
            .map_err(|e| println!("Err: {:?}", e))
            .is_ok());

        // Delete before timestamp
        let delete_res = ctx
            .repos
            .event_reminders_generation_jobs
            .delete_all_before(jobs[1].timestamp)
            .await;
        assert_eq!(delete_res.len(), 2);
        assert_eq!(delete_res[0], jobs[0]);
        assert_eq!(delete_res[1], jobs[1]);
    }
}
