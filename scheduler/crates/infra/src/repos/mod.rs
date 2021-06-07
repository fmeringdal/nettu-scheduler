mod account;
mod calendar;
mod event;
// mod kv;
mod schedule;
mod service;
mod service_user;
mod shared;
mod user;

use account::{IAccountRepo, PostgresAccountRepo};
use calendar::{ICalendarRepo, PostgresCalendarRepo};
use event::{
    IEventRemindersExpansionJobsRepo, IEventRepo, IReminderRepo,
    PostgresEventReminderExpansionJobsRepo, PostgresEventRepo, PostgresReminderRepo,
};
use schedule::{IScheduleRepo, PostgresScheduleRepo};
use service::{IServiceRepo, PostgresServiceRepo};
use service_user::{IServiceUserRepo, PostgresServiceUserRepo};
pub use shared::query_structs::*;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;
use user::{IUserRepo, PostgresUserRepo};

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
}
