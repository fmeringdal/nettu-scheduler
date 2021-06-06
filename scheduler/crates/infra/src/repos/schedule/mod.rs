mod postgres;

use crate::MetadataFindQuery;
use nettu_scheduler_domain::{Schedule, ID};
pub use postgres::PostgresScheduleRepo;

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn find(&self, schedule_id: &ID) -> Option<Schedule>;
    async fn find_many(&self, schedule_ids: &[ID]) -> Vec<Schedule>;
    async fn find_by_user(&self, user_id: &ID) -> Vec<Schedule>;
    async fn delete(&self, schedule_id: &ID) -> Option<Schedule>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Schedule>;
}

// #[cfg(test)]
// mod tests {
//     use crate::{setup_context, NettuContext};
//     use chrono_tz::US::Pacific;

//     use nettu_scheduler_domain::{Entity, Schedule, ID};

//     async fn create_contexts() -> Vec<NettuContext> {
//         vec![NettuContext::create_inmemory(), setup_context().await]
//     }

//     #[tokio::test]
//     async fn create_and_delete() {
//         for ctx in create_contexts().await {
//             let user_id = ID::default();
//             let account_id = ID::default();

//             let schedule = Schedule::new(user_id, account_id, &Pacific);

//             // Insert
//             assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());

//             // Different find methods
//             let res = ctx.repos.schedules.find(&schedule.id).await.unwrap();
//             assert!(res.eq(&schedule));
//             let res = ctx
//                 .repos
//                 .schedules
//                 .find_many(&vec![schedule.id.clone()])
//                 .await;
//             assert_eq!(res.len(), 1);
//             assert!(res[0].eq(&schedule));

//             // Delete
//             let res = ctx.repos.schedules.delete(&schedule.id).await;
//             assert!(res.is_some());
//             assert!(res.unwrap().eq(&schedule));

//             // Find
//             assert!(ctx.repos.schedules.find(&schedule.id).await.is_none());

//             // Insert again
//             assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());
//             // Delete by user
//             ctx.repos
//                 .schedules
//                 .delete_by_user(&schedule.user_id)
//                 .await
//                 .expect("Delete by user");
//             assert!(ctx.repos.schedules.find(&schedule.id).await.is_none());
//         }
//     }

//     #[tokio::test]
//     async fn update() {
//         for ctx in create_contexts().await {
//             let user_id = ID::default();
//             let account_id = ID::default();
//             let mut schedule = Schedule::new(user_id, account_id, &Pacific);

//             // Insert
//             assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());

//             assert_eq!(schedule.rules.len(), 7);
//             schedule.rules = Vec::new();

//             // Save
//             assert!(ctx.repos.schedules.save(&schedule).await.is_ok());

//             // Find
//             assert!(ctx
//                 .repos
//                 .schedules
//                 .find(&schedule.id)
//                 .await
//                 .unwrap()
//                 .rules
//                 .is_empty());
//         }
//     }
// }
