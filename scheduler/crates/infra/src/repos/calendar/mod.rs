mod postgres;

use super::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Calendar, ID};
pub use postgres::PostgresCalendarRepo;

#[async_trait::async_trait]
pub trait ICalendarRepo: Send + Sync {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn find(&self, calendar_id: &ID) -> Option<Calendar>;
    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar>;
    async fn delete(&self, calendar_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Calendar>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Calendar, Entity, User};

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);

        // Insert
        assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

        // Different find methods
        let res = ctx.repos.calendars.find(&calendar.id).await.unwrap();
        assert!(res.eq(&calendar));
        let res = ctx.repos.calendars.find_by_user(&user.id).await;
        assert!(res[0].eq(&calendar));

        // Delete
        let res = ctx.repos.calendars.delete(&calendar.id).await;
        assert!(res.is_ok());

        // Find
        assert!(ctx.repos.calendars.find(&calendar.id).await.is_none());
    }

    #[tokio::test]
    async fn update() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let mut calendar = Calendar::new(&user.id, &account.id);

        // Insert
        assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());
        calendar.settings.week_start = calendar.settings.week_start.succ();

        // Save
        assert!(ctx.repos.calendars.save(&calendar).await.is_ok());

        let updated_calendar = ctx.repos.calendars.find(&calendar.id).await.unwrap();
        assert_eq!(
            updated_calendar.settings.week_start,
            calendar.settings.week_start
        );
    }

    #[tokio::test]
    async fn delete_by_user() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone());
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id);

        // Insert
        assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

        // Delete
        let res = ctx.repos.users.delete(&user.id).await;
        assert!(res.is_some());

        // Find
        assert!(ctx.repos.calendars.find(&calendar.id).await.is_none());
    }
}
