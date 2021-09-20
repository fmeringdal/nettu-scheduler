mod postgres;

use nettu_scheduler_domain::{SyncedCalendarEvent, ID};
pub use postgres::PostgresEventSyncedRepo;

#[async_trait::async_trait]
pub trait IEventSyncedRepo: Send + Sync {
    async fn insert(&self, e: &SyncedCalendarEvent) -> anyhow::Result<()>;
    async fn find_by_event(&self, event_id: &ID) -> anyhow::Result<Vec<SyncedCalendarEvent>>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, AccountIntegration, Calendar, CalendarEvent, IntegrationProvider, SyncedCalendar,
        SyncedCalendarEvent, User, UserIntegration,
    };

    #[tokio::test]
    async fn test_event_synced_repo() {
        let ctx = setup_context().await;

        let account = Account::new();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");

        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.expect("To insert user");

        for provider in [IntegrationProvider::Google, IntegrationProvider::Outlook] {
            let acc_integration = AccountIntegration {
                account_id: account.id.clone(),
                client_id: "".into(),
                client_secret: "".into(),
                redirect_uri: "".into(),
                provider: provider.clone(),
            };
            ctx.repos
                .account_integrations
                .insert(&acc_integration)
                .await
                .expect("To insert account integration");

            let user_integration = UserIntegration {
                access_token: "".into(),
                access_token_expires_ts: 0,
                refresh_token: "".into(),
                account_id: account.id.clone(),
                user_id: user.id.clone(),
                provider,
            };
            ctx.repos
                .user_integrations
                .insert(&user_integration)
                .await
                .expect("To insert user integration");
        }

        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos
            .calendars
            .insert(&calendar)
            .await
            .expect("To insert calendar");

        for provider in [IntegrationProvider::Google, IntegrationProvider::Outlook] {
            let sync_calendar = SyncedCalendar {
                calendar_id: calendar.id.clone(),
                ext_calendar_id: "".into(),
                provider,
                user_id: user.id.clone(),
            };
            assert!(ctx
                .repos
                .calendar_synced
                .insert(&sync_calendar)
                .await
                .is_ok());
        }

        let e = CalendarEvent {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            ..Default::default()
        };
        assert!(ctx.repos.events.insert(&e).await.is_ok());

        for provider in [IntegrationProvider::Google, IntegrationProvider::Outlook] {
            let sync_event = SyncedCalendarEvent {
                calendar_id: calendar.id.clone(),
                event_id: e.id.clone(),
                ext_calendar_id: "".into(),
                ext_event_id: "".into(),
                provider,
                user_id: user.id.clone(),
            };
            assert!(ctx.repos.event_synced.insert(&sync_event).await.is_ok());
        }

        let synced_events = ctx
            .repos
            .event_synced
            .find_by_event(&e.id)
            .await
            .expect("To find synced calendar event");
        assert_eq!(synced_events.len(), 2);
        assert_eq!(synced_events[0].event_id, e.id);
        assert_eq!(synced_events[1].event_id, e.id);
        assert!(synced_events
            .iter()
            .find(|c| c.provider == IntegrationProvider::Google)
            .is_some());
        assert!(synced_events
            .iter()
            .find(|c| c.provider == IntegrationProvider::Outlook)
            .is_some());

        // Deleting the sync calendar also deletes all the corresponding sync events for that calendar
        let sync_calendar = SyncedCalendar {
            calendar_id: calendar.id.clone(),
            ext_calendar_id: "".into(),
            provider: IntegrationProvider::Google,
            user_id: user.id.clone(),
        };
        assert!(ctx
            .repos
            .calendar_synced
            .delete(&sync_calendar)
            .await
            .is_ok());

        let synced_events = ctx
            .repos
            .event_synced
            .find_by_event(&e.id)
            .await
            .expect("To find synced calendar event");
        assert_eq!(synced_events.len(), 1);
        assert_eq!(synced_events[0].provider, IntegrationProvider::Outlook);

        // And now delete outlook calendar sync
        let sync_calendar = SyncedCalendar {
            calendar_id: calendar.id.clone(),
            ext_calendar_id: "".into(),
            provider: IntegrationProvider::Outlook,
            user_id: user.id.clone(),
        };
        assert!(ctx
            .repos
            .calendar_synced
            .delete(&sync_calendar)
            .await
            .is_ok());
        let synced_events = ctx
            .repos
            .event_synced
            .find_by_event(&e.id)
            .await
            .expect("To find synced calendar event");
        assert!(synced_events.is_empty());
    }
}
