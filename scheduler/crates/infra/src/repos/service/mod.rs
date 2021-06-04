mod inmemory;
mod mongo;

pub use inmemory::InMemoryServiceRepo;
pub use mongo::MongoServiceRepo;
use nettu_scheduler_domain::{Service, ID};

use super::shared::query_structs::MetadataFindQuery;

#[async_trait::async_trait]
pub trait IServiceRepo: Send + Sync {
    async fn insert(&self, service: &Service) -> anyhow::Result<()>;
    async fn save(&self, service: &Service) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID) -> Option<Service>;
    async fn delete(&self, service_id: &ID) -> Option<Service>;
    async fn remove_calendar_from_services(&self, calendar_id: &ID) -> anyhow::Result<()>;
    async fn remove_schedule_from_services(&self, schedule_id: &ID) -> anyhow::Result<()>;
    async fn remove_user_from_services(&self, user_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service>;
}

#[cfg(test)]
mod tests {
    use crate::{setup_context, NettuContext};
    use nettu_scheduler_domain::{Service, ServiceResource, TimePlan, ID};

    /// Creates inmemory and mongo context when mongo is running,
    /// otherwise it will create two inmemory
    async fn create_contexts() -> Vec<NettuContext> {
        vec![NettuContext::create_inmemory().await, setup_context().await]
    }

    #[tokio::test]
    async fn create_and_delete() {
        for ctx in create_contexts().await {
            let account_id = ID::default();
            let service = Service::new(account_id);

            // Insert
            assert!(ctx.repos.service_repo.insert(&service).await.is_ok());

            // Get by id
            let mut service = ctx
                .repos
                .service_repo
                .find(&service.id)
                .await
                .expect("To get service");

            let user_id = ID::default();
            let calendar_id = ID::default();
            let timeplan = TimePlan::Empty;
            let resource =
                ServiceResource::new(user_id.clone(), timeplan, vec![calendar_id.clone()]);
            service.add_user(resource);

            ctx.repos
                .service_repo
                .save(&service)
                .await
                .expect("To save service");

            let service = ctx
                .repos
                .service_repo
                .find(&service.id)
                .await
                .expect("To get service");
            assert_eq!(service.users.len(), 1);
            assert_eq!(service.users[0].busy, vec![calendar_id.clone()]);

            ctx.repos
                .service_repo
                .remove_calendar_from_services(&calendar_id)
                .await
                .expect("To remove calendar from services");

            let mut service = ctx
                .repos
                .service_repo
                .find(&service.id)
                .await
                .expect("To get service");
            assert_eq!(service.users.len(), 1);
            println!("Service user: {:?}", service.users);
            assert!(service.users[0].busy.is_empty());

            let mut user = service.find_user_mut(&user_id).expect("To find user");
            user.availability = TimePlan::Calendar(calendar_id.clone());

            ctx.repos
                .service_repo
                .save(&service)
                .await
                .expect("To save service");

            ctx.repos
                .service_repo
                .remove_calendar_from_services(&calendar_id)
                .await
                .expect("To remove calendar from services");

            let service = ctx
                .repos
                .service_repo
                .find(&service.id)
                .await
                .expect("To get service");
            assert_eq!(service.users.len(), 1);
            assert!(service.users[0].busy.is_empty());
            assert_eq!(service.users[0].availability, TimePlan::Empty);

            ctx.repos
                .service_repo
                .remove_user_from_services(&user_id)
                .await
                .expect("To remove user from services");

            let service = ctx
                .repos
                .service_repo
                .find(&service.id)
                .await
                .expect("To get service");
            assert!(service.users.is_empty());

            ctx.repos
                .service_repo
                .delete(&service.id)
                .await
                .expect("To delete service");

            assert!(ctx.repos.service_repo.find(&service.id).await.is_none());
        }
    }
}
