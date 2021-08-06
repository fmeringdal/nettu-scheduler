mod postgres;

use nettu_scheduler_domain::{BusyCalendar, IntegrationProvider, ID};
pub use postgres::PostgresServiceUseBusyCalendarRepo;

#[derive(Debug, Clone)]
pub struct BusyCalendarIdentifier {
    pub service_id: ID,
    pub user_id: ID,
    pub calendar_id: ID,
}

#[derive(Debug, Clone)]
pub struct ExternalBusyCalendarIdentifier {
    pub service_id: ID,
    pub user_id: ID,
    pub ext_calendar_id: String,
    pub provider: IntegrationProvider,
}

#[async_trait::async_trait]
pub trait IServiceUserBusyCalendarRepo: Send + Sync {
    async fn exists(&self, input: BusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn exists_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<bool>;
    async fn insert(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn insert_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete(&self, input: BusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn delete_ext(&self, input: ExternalBusyCalendarIdentifier) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID, user_id: &ID) -> anyhow::Result<Vec<BusyCalendar>>;
}

#[cfg(test)]
mod tests {
    use crate::{setup_context, BusyCalendarIdentifier, ExternalBusyCalendarIdentifier};
    use nettu_scheduler_domain::{
        Account, AccountIntegration, Calendar, IntegrationProvider, Service, ServiceResource,
        TimePlan, User, UserIntegration,
    };

    #[tokio::test]
    async fn test_service_user_busy_calendars() {
        let ctx = setup_context().await;

        let account = Account::new();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.expect("To insert user");
        let service = Service::new(account.id.clone());
        ctx.repos.services.insert(&service).await.unwrap();
        let service_user =
            ServiceResource::new(user.id.clone(), service.id.clone(), TimePlan::Empty);
        assert!(ctx.repos.service_users.insert(&service_user).await.is_ok());

        for provider in [IntegrationProvider::Google, IntegrationProvider::Outlook] {
            let acc_integration = AccountIntegration {
                account_id: account.id.clone(),
                client_id: "".into(),
                client_secret: "".into(),
                redirect_uri: "".into(),
                provider: provider.clone(),
            };
            assert!(ctx
                .repos
                .account_integrations
                .insert(&acc_integration)
                .await
                .is_ok());

            let user_integration = UserIntegration {
                access_token: "".into(),
                access_token_expires_ts: 0,
                refresh_token: "".into(),
                account_id: account.id.clone(),
                user_id: user.id.clone(),
                provider: provider.clone(),
            };
            assert!(ctx
                .repos
                .user_integrations
                .insert(&user_integration)
                .await
                .is_ok());
        }

        let calendar = Calendar::new(&user.id, &account.id);
        assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

        // Insert nettu busy calendar
        let busy = BusyCalendarIdentifier {
            calendar_id: calendar.id.clone(),
            service_id: service.id.clone(),
            user_id: user.id.clone(),
        };
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .insert(busy.clone())
            .await
            .is_ok());
        // Double insert should be error
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .insert(busy.clone())
            .await
            .is_err());
        // It should find the busy calendar after insert
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .exists(busy.clone())
            .await
            .expect("To check if busy calendar exists"));
        // It should remove a busy calendar
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .delete(busy.clone())
            .await
            .is_ok());
        // It should NOT find the busy calendar after delete
        assert!(!ctx
            .repos
            .service_user_busy_calendars
            .exists(busy.clone())
            .await
            .expect("To check if busy calendar exists"));

        // Insert nettu busy calendar that does not exist
        let bad_busy = BusyCalendarIdentifier {
            calendar_id: Default::default(),
            service_id: service.id.clone(),
            user_id: user.id.clone(),
        };
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .insert(bad_busy.clone())
            .await
            .is_err());
        // Query if that exists should return false
        assert!(!ctx
            .repos
            .service_user_busy_calendars
            .exists(bad_busy.clone())
            .await
            .expect("To check if busy calendar exists"));

        // ------------------------------------
        // Insert external busy calendar
        let busy = ExternalBusyCalendarIdentifier {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            ext_calendar_id: "".into(),
            provider: IntegrationProvider::Google,
        };
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .insert_ext(busy.clone())
            .await
            .is_ok());
        // Double insert should be error
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .insert_ext(busy.clone())
            .await
            .is_err());
        // It should find the busy calendar after insert
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .exists_ext(busy.clone())
            .await
            .expect("To check if busy calendar exists"));
        // It should remove a busy calendar
        assert!(ctx
            .repos
            .service_user_busy_calendars
            .delete_ext(busy.clone())
            .await
            .is_ok());
        // It should NOT find the busy calendar after delete
        assert!(!ctx
            .repos
            .service_user_busy_calendars
            .exists_ext(busy.clone())
            .await
            .expect("To check if busy calendar exists"));

        // External busy calendar that does not exist
        let bad_busy = ExternalBusyCalendarIdentifier {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            ext_calendar_id: "21412412".into(),
            provider: IntegrationProvider::Google,
        };
        // Query if that exists should return false
        assert!(!ctx
            .repos
            .service_user_busy_calendars
            .exists_ext(bad_busy.clone())
            .await
            .expect("To check if busy calendar exists"));
    }
}
