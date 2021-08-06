mod postgres;

use nettu_scheduler_domain::ID;
pub use postgres::PostgresReservationRepo;

#[async_trait::async_trait]
pub trait IReservationRepo: Send + Sync {
    async fn increment(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<()>;
    async fn decrement(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<()>;
    async fn count(&self, service_id: &ID, timestamp: i64) -> anyhow::Result<usize>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Service};

    #[tokio::test]
    async fn test_reservations_repo() {
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

        // Is null before inserting
        let count = ctx
            .repos
            .reservations
            .count(&service.id, 0)
            .await
            .expect("To get reservations count");
        assert_eq!(count, 0);

        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, 0)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, 1)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, 2)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service2.id, 1)
            .await
            .is_ok());
        let count = ctx
            .repos
            .reservations
            .count(&service.id, 1)
            .await
            .expect("To get reservations count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_delete_reservation() {
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

        let timestamp = 10;

        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, timestamp)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, timestamp)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, timestamp)
            .await
            .is_ok());
        assert!(ctx
            .repos
            .reservations
            .increment(&service.id, timestamp)
            .await
            .is_ok());
        let count = ctx
            .repos
            .reservations
            .count(&service.id, timestamp)
            .await
            .expect("To get reservations count");
        assert_eq!(count, 4);

        // Delete one reservation
        assert!(ctx
            .repos
            .reservations
            .decrement(&service.id, timestamp)
            .await
            .is_ok());

        // Now there should only be three
        let count = ctx
            .repos
            .reservations
            .count(&service.id, timestamp)
            .await
            .expect("To get reservations count");

        assert_eq!(count, 3);
    }
}
