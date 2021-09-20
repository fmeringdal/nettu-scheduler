mod postgres;

use nettu_scheduler_domain::{Reminder, ID};
pub use postgres::PostgresReminderRepo;

#[async_trait::async_trait]
pub trait IReminderRepo: Send + Sync {
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()>;
    async fn init_version(&self, event_id: &ID) -> anyhow::Result<i64>;
    async fn inc_version(&self, event_id: &ID) -> anyhow::Result<i64>;
    async fn delete_all_before(&self, before: i64) -> Vec<Reminder>;
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
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            start_ts: 1000 * 60 * 60,
            user_id: user.id.clone(),
            ..Default::default()
        };
        ctx.repos.events.insert(&event).await.unwrap();

        let version = ctx
            .repos
            .reminders
            .init_version(&event.id)
            .await
            .expect("To create reminder version");

        let reminders = vec![
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                version,
                remind_at: 1,
                identifier: "".into(),
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                version,
                remind_at: 2,
                identifier: "".into(),
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                version,
                remind_at: 3,
                identifier: "".into(),
            },
            Reminder {
                account_id: account.id.clone(),
                event_id: event.id.clone(),
                version,
                remind_at: 4,
                identifier: "".into(),
            },
        ];
        assert!(ctx.repos.reminders.bulk_insert(&reminders).await.is_ok());

        // Delete before timestamp
        let delete_res = ctx
            .repos
            .reminders
            .delete_all_before(reminders[1].remind_at)
            .await;
        assert_eq!(delete_res.len(), 2);
        assert_eq!(delete_res[0], reminders[0]);
        assert_eq!(delete_res[1], reminders[1]);

        // Inc version number
        let new_e3_v = ctx
            .repos
            .reminders
            .inc_version(&reminders[2].event_id)
            .await
            .expect("To increment reminder version");
        assert_eq!(new_e3_v, reminders[2].version + 1);
        let delete_res = ctx
            .repos
            .reminders
            .delete_all_before(reminders[3].remind_at)
            .await;
        // Reminders has been deleted because there is a new version now
        assert_eq!(delete_res.len(), 0);
    }
}
