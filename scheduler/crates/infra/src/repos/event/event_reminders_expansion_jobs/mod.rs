mod postgres;

use nettu_scheduler_domain::{EventRemindersExpansionJob, ID};
pub use postgres::PostgresEventReminderExpansionJobsRepo;

use crate::repos::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IEventRemindersExpansionJobsRepo: Send + Sync {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()>;
    async fn delete_all_before(&self, before: i64) -> Vec<EventRemindersExpansionJob>;
    async fn delete_by_event(&self, event_id: &ID) -> anyhow::Result<DeleteResult>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, Calendar, CalendarEvent, EventRemindersExpansionJob, Reminder, User,
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
        let event = CalendarEvent {
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
            reminder: None,
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        ctx.repos.events.insert(&event).await.unwrap();

        let jobs = vec![
            EventRemindersExpansionJob {
                id: Default::default(),
                event_id: event.id.clone(),
                timestamp: 1,
            },
            EventRemindersExpansionJob {
                id: Default::default(),
                event_id: event.id.clone(),
                timestamp: 2,
            },
            EventRemindersExpansionJob {
                id: Default::default(),
                event_id: event.id.clone(),
                timestamp: 3,
            },
            EventRemindersExpansionJob {
                id: Default::default(),
                event_id: event.id.clone(),
                timestamp: 4,
            },
        ];
        assert!(ctx
            .repos
            .event_reminders_expansion_jobs
            .bulk_insert(&jobs)
            .await
            .is_ok());

        // Delete before timestamp
        let delete_res = ctx
            .repos
            .event_reminders_expansion_jobs
            .delete_all_before(jobs[1].timestamp)
            .await;
        assert_eq!(delete_res.len(), 2);
        assert_eq!(delete_res[0].id, jobs[0].id);
        assert_eq!(delete_res[1].id, jobs[1].id);

        // Delete by event
        let delete_res = ctx
            .repos
            .event_reminders_expansion_jobs
            .delete_by_event(&event.id)
            .await;
        assert!(delete_res.is_ok());
        assert_eq!(delete_res.unwrap().deleted_count, 2);
    }
}
