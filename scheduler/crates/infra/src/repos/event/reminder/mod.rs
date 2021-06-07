mod postgres;

use nettu_scheduler_domain::{Reminder, ID};
pub use postgres::PostgresReminderRepo;

use crate::repos::shared::repo::DeleteResult;

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()>;
    async fn find_by_event_and_priority(&self, event_id: &ID, priority: i64) -> Option<Reminder>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
    async fn delete_by_events(&self, event_ids: &[ID]) -> anyhow::Result<DeleteResult>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Calendar, CalendarEvent, Reminder, User};

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
            is_service: false,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        ctx.repos.events.insert(&event).await.unwrap();

        let reminders = vec![
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                id: Default::default(),
                priority: 0,
                remind_at: 1,
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                id: Default::default(),
                priority: 2,
                remind_at: 2,
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                id: Default::default(),
                priority: 0,
                remind_at: 3,
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                id: Default::default(),
                priority: 0,
                remind_at: 4,
            },
        ];
        assert!(ctx.repos.reminders.bulk_insert(&reminders).await.is_ok());

        // Find
        let find_res = ctx
            .repos
            .reminders
            .find_by_event_and_priority(&event.id, reminders[1].priority)
            .await;
        assert!(find_res.is_some());
        assert_eq!(find_res.unwrap().id, reminders[1].id);

        // Delete before timestamp
        let delete_res = ctx
            .repos
            .reminders
            .delete_all_before(reminders[1].remind_at)
            .await;
        assert_eq!(delete_res.len(), 2);
        assert_eq!(delete_res[0].id, reminders[0].id);
        assert_eq!(delete_res[1].id, reminders[1].id);

        // Delete by event
        let delete_res = ctx
            .repos
            .reminders
            .delete_by_events(&[event.id.clone()])
            .await;
        assert!(delete_res.is_ok());
        assert_eq!(delete_res.unwrap().deleted_count, 2);
    }
}
