mod inmemory;
mod mongo;

use crate::repos::shared::repo::DeleteResult;
pub use inmemory::InMemoryCalendarRepo;
pub use mongo::MongoCalendarRepo;
use nettu_scheduler_domain::{Calendar, ID};

#[async_trait::async_trait]
pub trait ICalendarRepo: Send + Sync {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn find(&self, calendar_id: &ID) -> Option<Calendar>;
    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar>;
    async fn delete(&self, calendar_id: &ID) -> Option<Calendar>;
    async fn delete_by_user(&self, user_id: &ID) -> anyhow::Result<DeleteResult>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Calendar, Entity, ID};

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await;
        let user_id = ID::default();
        let calendar = Calendar::new(&user_id);

        // Insert
        assert!(ctx.repos.calendar_repo.insert(&calendar).await.is_ok());

        // Different find methods
        let res = ctx.repos.calendar_repo.find(&calendar.id).await.unwrap();
        assert!(res.eq(&calendar));
        let res = ctx.repos.calendar_repo.find_by_user(&user_id).await;
        assert!(res[0].eq(&calendar));

        // Delete
        let res = ctx.repos.calendar_repo.delete(&calendar.id).await;
        assert!(res.is_some());
        assert!(res.unwrap().eq(&calendar));

        // Find
        assert!(ctx.repos.calendar_repo.find(&calendar.id).await.is_none());
    }

    #[tokio::test]
    async fn update() {
        let ctx = setup_context().await;
        let user_id = ID::default();
        let mut calendar = Calendar::new(&user_id);

        // Insert
        assert!(ctx.repos.calendar_repo.insert(&calendar).await.is_ok());

        calendar.settings.week_start += 1;

        // Save
        assert!(ctx.repos.calendar_repo.save(&calendar).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .calendar_repo
            .find(&calendar.id)
            .await
            .unwrap()
            .eq(&calendar));
    }

    #[tokio::test]
    async fn delete_by_user() {
        let ctx = setup_context().await;
        let user_id = ID::default();
        let calendar = Calendar::new(&user_id);

        // Insert
        assert!(ctx.repos.calendar_repo.insert(&calendar).await.is_ok());

        // Delete
        let res = ctx.repos.calendar_repo.delete_by_user(&user_id).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().deleted_count, 1);

        // Find
        assert!(ctx.repos.calendar_repo.find(&calendar.id).await.is_none());
    }
}
