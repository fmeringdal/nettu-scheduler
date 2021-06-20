mod postgres;

use nettu_scheduler_domain::{ServiceReservation, ID};
pub use postgres::PostgresReservationRepo;

// On create booking intent when service is group
// - Get number of reservations overlapping with that
// - Insert reservation
// - Return if service event needs to be created -> when reserveations == max count with the newly created
// On update max count
// - If less than do nothing ?
// - If more then delete all service events

#[async_trait::async_trait]
pub trait IReservationRepo: Send + Sync {
    async fn insert(&self, reservation: &ServiceReservation) -> anyhow::Result<()>;
    async fn find_in_range(
        &self,
        service_id: &ID,
        min_ts: i64,
        max_ts: i64,
    ) -> Vec<ServiceReservation>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Service};

    #[tokio::test]
    async fn test_metadata_query() {
        let ctx = setup_context().await;

        let account = Account::new();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");
        let service = Service::new(account.id.clone());
        ctx.repos
            .services
            .insert(&service)
            .await
            .expect("To insert service");
        let service2 = Service::new(account.id.clone());
        ctx.repos
            .services
            .insert(&service2)
            .await
            .expect("To insert service");

        let reservation1 = ServiceReservation {
            id: Default::default(),
            service_id: service.id.clone(),
            timestamp: 0,
        };
        let reservation2 = ServiceReservation {
            id: Default::default(),
            service_id: service.id.clone(),
            timestamp: 1,
        };
        let reservation3 = ServiceReservation {
            id: Default::default(),
            service_id: service2.id.clone(),
            timestamp: 1,
        };
        let reservation4 = ServiceReservation {
            id: Default::default(),
            service_id: service.id.clone(),
            timestamp: 2,
        };
        assert!(ctx.repos.reservations.insert(&reservation1).await.is_ok());
        assert!(ctx.repos.reservations.insert(&reservation2).await.is_ok());
        assert!(ctx.repos.reservations.insert(&reservation3).await.is_ok());
        assert!(ctx.repos.reservations.insert(&reservation4).await.is_ok());
        let res_in_range = ctx
            .repos
            .reservations
            .find_in_range(&service.id, 0, 2)
            .await;
        assert_eq!(res_in_range.len(), 1);
        assert_eq!(res_in_range[0].id, reservation2.id);
    }
}
