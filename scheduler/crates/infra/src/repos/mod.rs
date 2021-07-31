mod account;
mod account_integrations;
mod calendar;
mod calendar_synced;
mod event;
// mod kv;
mod reservation;
mod schedule;
mod service;
mod service_user;
mod service_user_busy_calendars;
mod shared;
pub(crate) mod user;
mod user_integrations;

use account::{IAccountRepo, PostgresAccountRepo};
use account_integrations::{IAccountIntegrationRepo, PostgresAccountIntegrationRepo};
use calendar::{ICalendarRepo, PostgresCalendarRepo};
use calendar_synced::{ICalendarSyncedRepo, PostgresCalendarSyncedRepo};
use event::{
    IEventRemindersExpansionJobsRepo, IEventRepo, IEventSyncedRepo, IReminderRepo,
    PostgresEventReminderExpansionJobsRepo, PostgresEventRepo, PostgresEventSyncedRepo,
    PostgresReminderRepo,
};
use reservation::{IReservationRepo, PostgresReservationRepo};
use schedule::{IScheduleRepo, PostgresScheduleRepo};
use service::{IServiceRepo, PostgresServiceRepo};
use service_user::{IServiceUserRepo, PostgresServiceUserRepo};
use service_user_busy_calendars::{
    IServiceUserBusyCalendarRepo, PostgresServiceUseBusyCalendarRepo,
};
pub use shared::query_structs::*;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;
use user::{IUserRepo, PostgresUserRepo};
use user_integrations::{IUserIntegrationRepo, PostgresUserIntegrationRepo};

#[derive(Clone)]
pub struct Repos {
    pub accounts: Arc<dyn IAccountRepo>,
    pub account_integrations: Arc<dyn IAccountIntegrationRepo>,
    pub calendars: Arc<dyn ICalendarRepo>,
    pub calendar_synced: Arc<dyn ICalendarSyncedRepo>,
    pub events: Arc<dyn IEventRepo>,
    pub event_reminders_expansion_jobs: Arc<dyn IEventRemindersExpansionJobsRepo>,
    pub event_synced: Arc<dyn IEventSyncedRepo>,
    pub schedules: Arc<dyn IScheduleRepo>,
    pub reminders: Arc<dyn IReminderRepo>,
    pub reservations: Arc<dyn IReservationRepo>,
    pub services: Arc<dyn IServiceRepo>,
    pub service_users: Arc<dyn IServiceUserRepo>,
    pub service_user_busy_calendars: Arc<dyn IServiceUserBusyCalendarRepo>,
    pub users: Arc<dyn IUserRepo>,
    pub user_integrations: Arc<dyn IUserIntegrationRepo>,
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
            account_integrations: Arc::new(PostgresAccountIntegrationRepo::new(pool.clone())),
            calendars: Arc::new(PostgresCalendarRepo::new(pool.clone())),
            calendar_synced: Arc::new(PostgresCalendarSyncedRepo::new(pool.clone())),
            events: Arc::new(PostgresEventRepo::new(pool.clone())),
            event_synced: Arc::new(PostgresEventSyncedRepo::new(pool.clone())),
            users: Arc::new(PostgresUserRepo::new(pool.clone())),
            user_integrations: Arc::new(PostgresUserIntegrationRepo::new(pool.clone())),
            services: Arc::new(PostgresServiceRepo::new(pool.clone())),
            service_users: Arc::new(PostgresServiceUserRepo::new(pool.clone())),
            service_user_busy_calendars: Arc::new(PostgresServiceUseBusyCalendarRepo::new(
                pool.clone(),
            )),
            schedules: Arc::new(PostgresScheduleRepo::new(pool.clone())),
            reminders: Arc::new(PostgresReminderRepo::new(pool.clone())),
            reservations: Arc::new(PostgresReservationRepo::new(pool.clone())),
            event_reminders_expansion_jobs: Arc::new(PostgresEventReminderExpansionJobsRepo::new(
                pool,
            )),
        })
    }
}
