mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod service_user;
mod shared;
mod user;

use account::{IAccountRepo, InMemoryAccountRepo, PostgresAccountRepo};
use calendar::{ICalendarRepo, InMemoryCalendarRepo, PostgresCalendarRepo};
use event::{
    IEventRemindersExpansionJobsRepo, IEventRepo, IReminderRepo,
    InMemoryEventRemindersExpansionJobsRepo, InMemoryEventRepo, InMemoryReminderRepo,
    PostgresEventReminderExpansionJobsRepo, PostgresEventRepo, PostgresReminderRepo,
};
use schedule::{IScheduleRepo, InMemoryScheduleRepo, PostgresScheduleRepo};
use service::{IServiceRepo, InMemoryServiceRepo, PostgresServiceRepo};
use service_user::{IServiceUserRepo, InMemoryServiceUserRepo, PostgresServiceUserRepo};
pub use shared::query_structs::*;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;
use user::{IUserRepo, InMemoryUserRepo, PostgresUserRepo};

#[derive(Clone)]
pub struct Repos {
    pub events: Arc<dyn IEventRepo>,
    pub calendars: Arc<dyn ICalendarRepo>,
    pub accounts: Arc<dyn IAccountRepo>,
    pub users: Arc<dyn IUserRepo>,
    pub services: Arc<dyn IServiceRepo>,
    pub service_users: Arc<dyn IServiceUserRepo>,
    pub schedules: Arc<dyn IScheduleRepo>,
    pub reminders: Arc<dyn IReminderRepo>,
    pub event_reminders_expansion_jobs: Arc<dyn IEventRemindersExpansionJobsRepo>,
}

impl Repos {
    pub async fn create_postgres(
        connection_string: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!("DB CHECKING CONNECTION ...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .expect("TO CONNECT TO POSTGRES");
        info!("DB CHECKING CONNECTION ... [done]");
        Ok(Self {
            accounts: Arc::new(PostgresAccountRepo::new(pool.clone())),
            calendars: Arc::new(PostgresCalendarRepo::new(pool.clone())),
            events: Arc::new(PostgresEventRepo::new(pool.clone())),
            users: Arc::new(PostgresUserRepo::new(pool.clone())),
            services: Arc::new(PostgresServiceRepo::new(pool.clone())),
            service_users: Arc::new(PostgresServiceUserRepo::new(pool.clone())),
            schedules: Arc::new(PostgresScheduleRepo::new(pool.clone())),
            reminders: Arc::new(PostgresReminderRepo::new(pool.clone())),
            event_reminders_expansion_jobs: Arc::new(PostgresEventReminderExpansionJobsRepo::new(
                pool,
            )),
        })
    }

    pub fn create_inmemory() -> Self {
        Self {
            events: Arc::new(InMemoryEventRepo::new()),
            calendars: Arc::new(InMemoryCalendarRepo::new()),
            accounts: Arc::new(InMemoryAccountRepo::new()),
            users: Arc::new(InMemoryUserRepo::new()),
            services: Arc::new(InMemoryServiceRepo::new()),
            service_users: Arc::new(InMemoryServiceUserRepo::new()),
            schedules: Arc::new(InMemoryScheduleRepo::new()),
            reminders: Arc::new(InMemoryReminderRepo::new()),
            event_reminders_expansion_jobs: Arc::new(InMemoryEventRemindersExpansionJobsRepo::new()),
        }
    }
}
