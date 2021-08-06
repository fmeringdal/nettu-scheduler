mod postgres;

use nettu_scheduler_domain::{SyncedCalendar, ID};
pub use postgres::PostgresCalendarSyncedRepo;

#[async_trait::async_trait]
pub trait ICalendarSyncedRepo: Send + Sync {
    async fn insert(&self, c: &SyncedCalendar) -> anyhow::Result<()>;
    async fn delete(&self, c: &SyncedCalendar) -> anyhow::Result<()>;
    async fn find_by_calendar(&self, calendar_id: &ID) -> anyhow::Result<Vec<SyncedCalendar>>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, AccountIntegration, Calendar, IntegrationProvider, SyncedCalendar, User,
        UserIntegration,
    };

    #[tokio::test]
    async fn test_calendar_synced_repo() {
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

        let synced_calendars = ctx
            .repos
            .calendar_synced
            .find_by_calendar(&calendar.id)
            .await
            .expect("To find synced calendars");
        assert_eq!(synced_calendars.len(), 2);
        assert_eq!(synced_calendars[0].calendar_id, calendar.id);
        assert_eq!(synced_calendars[1].calendar_id, calendar.id);
        assert!(synced_calendars
            .iter()
            .find(|c| c.provider == IntegrationProvider::Google)
            .is_some());
        assert!(synced_calendars
            .iter()
            .find(|c| c.provider == IntegrationProvider::Outlook)
            .is_some());

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
        assert!(ctx
            .repos
            .calendar_synced
            .delete(&sync_calendar)
            .await
            .is_err());

        // Find after delete
        let synced_calendars = ctx
            .repos
            .calendar_synced
            .find_by_calendar(&calendar.id)
            .await
            .expect("To find synced calendars");
        assert_eq!(synced_calendars.len(), 1);
        assert_eq!(synced_calendars[0].calendar_id, calendar.id);
        assert_eq!(synced_calendars[0].provider, IntegrationProvider::Outlook);
    }
}
