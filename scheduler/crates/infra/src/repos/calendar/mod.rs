mod inmemory;
mod postgres;

pub use inmemory::InMemoryCalendarRepo;
use nettu_scheduler_domain::{Calendar, ID};
pub use postgres::PostgresCalendarRepo;

use super::shared::query_structs::MetadataFindQuery;

#[async_trait::async_trait]
pub trait ICalendarRepo: Send + Sync {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()>;
    async fn find(&self, calendar_id: &ID) -> Option<Calendar>;
    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar>;
    async fn delete(&self, calendar_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Calendar>;
}

// #[cfg(test)]
// mod tests {
//     use crate::{setup_context, NettuContext};
//     use nettu_scheduler_domain::{Calendar, Entity, ID};

//     /// Creates inmemory and mongo context when mongo is running,
//     /// otherwise it will create two inmemory
//     async fn create_contexts() -> Vec<NettuContext> {
//         vec![NettuContext::create_inmemory(), setup_context().await]
//     }

//     #[tokio::test]
//     async fn create_and_delete() {
//         for ctx in create_contexts().await {
//             let user_id = ID::default();
//             let account_id = ID::default();
//             let calendar = Calendar::new(&user_id, &account_id);

//             // Insert
//             assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

//             // Different find methods
//             let res = ctx.repos.calendars.find(&calendar.id).await.unwrap();
//             assert!(res.eq(&calendar));
//             let res = ctx.repos.calendars.find_by_user(&user_id).await;
//             assert!(res[0].eq(&calendar));

//             // Delete
//             let res = ctx.repos.calendars.delete(&calendar.id).await;
//             assert!(res.is_ok());

//             // Find
//             assert!(ctx.repos.calendars.find(&calendar.id).await.is_none());
//         }
//     }

//     #[tokio::test]
//     async fn update() {
//         for ctx in create_contexts().await {
//             let user_id = ID::default();
//             let account_id = ID::default();
//             let mut calendar = Calendar::new(&user_id, &account_id);

//             // Insert
//             assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

//             calendar.settings.week_start += 1;

//             // Save
//             assert!(ctx.repos.calendars.save(&calendar).await.is_ok());

//             // Find
//             assert!(ctx
//                 .repos
//                 .calendars
//                 .find(&calendar.id)
//                 .await
//                 .unwrap()
//                 .eq(&calendar));
//         }
//     }

//     #[tokio::test]
//     async fn delete_by_user() {
//         for ctx in create_contexts().await {
//             let user_id = ID::default();
//             let account_id = ID::default();
//             let calendar = Calendar::new(&user_id, &account_id);

//             // Insert
//             assert!(ctx.repos.calendars.insert(&calendar).await.is_ok());

//             // Delete
//             let res = ctx.repos.calendars.delete_by_user(&user_id).await;
//             assert!(res.is_ok());
//             assert_eq!(res.unwrap().deleted_count, 1);

//             // Find
//             assert!(ctx.repos.calendars.find(&calendar.id).await.is_none());
//         }
//     }
// }
