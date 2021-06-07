mod postgres;

use nettu_scheduler_domain::{ServiceResource, ID};
pub use postgres::PostgresServiceUserRepo;
pub use postgres::ServiceUserRaw;

#[async_trait::async_trait]
pub trait IServiceUserRepo: Send + Sync {
    async fn insert(&self, user: &ServiceResource) -> anyhow::Result<()>;
    async fn save(&self, user: &ServiceResource) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID, user_id: &ID) -> Option<ServiceResource>;
    async fn delete(&self, service_id: &ID, user_uid: &ID) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{
        Account, Calendar, Entity, Service, ServiceResource, TimePlan, User,
    };

    #[tokio::test]
    async fn crud() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let service = Service::new(account.id.clone());
        ctx.repos.services.insert(&service).await.unwrap();

        let service_user =
            ServiceResource::new(user.id.clone(), service.id.clone(), TimePlan::Empty, vec![]);
        // Insert
        assert!(ctx.repos.service_users.insert(&service_user).await.is_ok());

        // Find
        let res = ctx
            .repos
            .service_users
            .find(&service.id, &user.id)
            .await
            .unwrap();
        assert!(res.eq(&service_user));

        // Update
        let calendar = Calendar::new(&user.id, &account.id);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        let mut service_user = res;
        service_user.buffer_after = 60;
        service_user.availability = TimePlan::Calendar(calendar.id.clone());
        service_user.busy = vec![calendar.id.clone()];
        assert!(ctx.repos.service_users.save(&service_user).await.is_ok());

        let updated_service_user = ctx
            .repos
            .service_users
            .find(&service.id, &user.id)
            .await
            .unwrap();
        assert_eq!(updated_service_user.buffer_after, service_user.buffer_after);
        assert_eq!(updated_service_user.busy, service_user.busy);
        assert_eq!(updated_service_user.user_id, service_user.user_id);
        assert_eq!(updated_service_user.service_id, service_user.service_id);

        // Delete
        assert!(ctx
            .repos
            .service_users
            .delete(&service.id, &user.id)
            .await
            .is_ok());

        // Find after delete
        assert!(ctx
            .repos
            .service_users
            .find(&service.id, &user.id)
            .await
            .is_none());
    }
}
