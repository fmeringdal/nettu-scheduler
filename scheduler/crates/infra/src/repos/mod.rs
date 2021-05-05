mod account;
mod calendar;
mod event;
// mod kv;
mod schedule;
mod service;
mod shared;
mod user;

use account::{IAccountRepo, InMemoryAccountRepo, MongoAccountRepo};
use calendar::{ICalendarRepo, InMemoryCalendarRepo, MongoCalendarRepo};
use event::{
    IEventRemindersExpansionJobsRepo, IEventRepo, IReminderRepo,
    InMemoryEventRemindersExpansionJobsRepo, InMemoryEventRepo, InMemoryReminderRepo,
    MongoEventRemindersExpansionsJobRepo, MongoEventRepo, MongoReminderRepo,
};
// use kv::{IKVRepo, InMemoryKVRepo, MongoKVRepo};
use mongodb::{options::ClientOptions, Client};
use schedule::{IScheduleRepo, InMemoryScheduleRepo, MongoScheduleRepo};
use service::{IServiceRepo, InMemoryServiceRepo, MongoServiceRepo};
use std::sync::Arc;
use tracing::info;
use user::{IUserRepo, InMemoryUserRepo, MongoUserRepo};

pub use mongodb::bson::oid::ObjectId;
pub use shared::query_structs::*;

#[derive(Clone)]
pub struct Repos {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub account_repo: Arc<dyn IAccountRepo>,
    pub user_repo: Arc<dyn IUserRepo>,
    // pub key_value_repo: Arc<dyn IKVRepo>,
    pub service_repo: Arc<dyn IServiceRepo>,
    pub schedule_repo: Arc<dyn IScheduleRepo>,
    pub reminder_repo: Arc<dyn IReminderRepo>,
    pub event_reminders_expansion_jobs_repo: Arc<dyn IEventRemindersExpansionJobsRepo>,
}

impl Repos {
    pub async fn create_mongodb(
        connection_string: &str,
        db_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(db_name);

        // This is needed to make sure that db is ready when opening server
        info!("DB CHECKING CONNECTION ...");
        db.collection("server-start")
            .insert_one(
                mongodb::bson::doc! {
                "server-start": 1
                },
                None,
            )
            .await?;
        info!("DB CHECKING CONNECTION ... [done]");
        Ok(Self {
            event_repo: Arc::new(MongoEventRepo::new(&db)),
            calendar_repo: Arc::new(MongoCalendarRepo::new(&db)),
            account_repo: Arc::new(MongoAccountRepo::new(&db)),
            user_repo: Arc::new(MongoUserRepo::new(&db)),
            // key_value_repo: Arc::new(MongoKVRepo::new(&db)),
            service_repo: Arc::new(MongoServiceRepo::new(&db)),
            schedule_repo: Arc::new(MongoScheduleRepo::new(&db)),
            reminder_repo: Arc::new(MongoReminderRepo::new(&db)),
            event_reminders_expansion_jobs_repo: Arc::new(
                MongoEventRemindersExpansionsJobRepo::new(&db),
            ),
        })
    }

    pub fn create_inmemory() -> Self {
        Self {
            event_repo: Arc::new(InMemoryEventRepo::new()),
            calendar_repo: Arc::new(InMemoryCalendarRepo::new()),
            account_repo: Arc::new(InMemoryAccountRepo::new()),
            user_repo: Arc::new(InMemoryUserRepo::new()),
            // key_value_repo: Arc::new(InMemoryKVRepo::new()),
            service_repo: Arc::new(InMemoryServiceRepo::new()),
            schedule_repo: Arc::new(InMemoryScheduleRepo::new()),
            reminder_repo: Arc::new(InMemoryReminderRepo::new()),
            event_reminders_expansion_jobs_repo: Arc::new(
                InMemoryEventRemindersExpansionJobsRepo::new(),
            ),
        }
    }
}
