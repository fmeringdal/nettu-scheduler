mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod shared;
mod user;

use std::sync::Arc;

use account::{AccountRepo, IAccountRepo, InMemoryAccountRepo};
use calendar::{CalendarRepo, ICalendarRepo, InMemoryCalendarRepo};
use chrono::Utc;
use event::{
    EventRepo, IEventRepo, IReminderRepo, InMemoryEventRepo, InMemoryReminderRepo, ReminderRepo,
};
use mongodb::{options::ClientOptions, Client};
use nettu_scheduler_utils::create_random_secret;
use schedule::{IScheduleRepo, InMemoryScheduleRepo, ScheduleRepo};
use service::{IServiceRepo, InMemoryServiceRepo, ServiceRepo};
use user::{IUserRepo, InMemoryUserRepo, UserRepo};

pub use mongodb::bson::oid::ObjectId;

#[derive(Clone)]
pub struct Repos {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
    pub account_repo: Arc<dyn IAccountRepo>,
    pub user_repo: Arc<dyn IUserRepo>,
    pub service_repo: Arc<dyn IServiceRepo>,
    pub schedule_repo: Arc<dyn IScheduleRepo>,
    pub reminder_repo: Arc<dyn IReminderRepo>,
}

impl Repos {
    pub async fn create_mongodb() -> Result<Self, Box<dyn std::error::Error>> {
        let client_options = ClientOptions::parse(
            &std::env::var("MONGODB_CONNECTION_STRING")
                .expect("Expected MONGODB_CONNECTION_STRING env to be present"),
        )
        .await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(
            &std::env::var("MONGODB_NAME").expect("Expected MONGODB_NAME env to be present"),
        );

        // This is needed to make sure that db is ready when opening server
        println!("DB CHECKING CONNECTION ...");
        db.collection("server-start")
            .insert_one(
                mongodb::bson::doc! {
                "server-start": 1
                },
                None,
            )
            .await?;
        println!("DB CHECKING CONNECTION ... [done]");
        Ok(Self {
            event_repo: Arc::new(EventRepo::new(&db)),
            calendar_repo: Arc::new(CalendarRepo::new(&db)),
            account_repo: Arc::new(AccountRepo::new(&db)),
            user_repo: Arc::new(UserRepo::new(&db)),
            service_repo: Arc::new(ServiceRepo::new(&db)),
            schedule_repo: Arc::new(ScheduleRepo::new(&db)),
            reminder_repo: Arc::new(ReminderRepo::new(&db)),
        })
    }

    pub fn create_inmemory() -> Self {
        println!("using inmemory database");
        Self {
            event_repo: Arc::new(InMemoryEventRepo::new()),
            calendar_repo: Arc::new(InMemoryCalendarRepo::new()),
            account_repo: Arc::new(InMemoryAccountRepo::new()),
            user_repo: Arc::new(InMemoryUserRepo::new()),
            service_repo: Arc::new(InMemoryServiceRepo::new()),
            schedule_repo: Arc::new(InMemoryScheduleRepo::new()),
            reminder_repo: Arc::new(InMemoryReminderRepo::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub create_account_secret_code: String,
}

impl Config {
    pub fn new() -> Self {
        let create_account_secret_code = match std::env::var("CREATE_ACCOUNT_SECRET_CODE") {
            Ok(code) => code,
            Err(_) => {
                println!("Did not find CREATE_ACCOUNT_SECRET_CODE environment variable. Going to create one.");
                let code = create_random_secret(16);
                println!(
                    "Secret code for creating accounts was generated and set to: {}",
                    code
                );
                code
            }
        };
        Self {
            create_account_secret_code,
        }
    }
}

pub trait ISys: Send + Sync {
    fn get_utc_timestamp(&self) -> i64;
}

pub struct MockSys {}
impl ISys for MockSys {
    fn get_utc_timestamp(&self) -> i64 {
        0 // 1970
    }
}

pub struct RealSys {}
impl ISys for RealSys {
    fn get_utc_timestamp(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

#[derive(Clone)]
pub struct Context {
    pub repos: Repos,
    pub config: Config,
    pub sys: Arc<dyn ISys>,
}

impl Context {
    pub fn create_inmemory() -> Self {
        Self {
            repos: Repos::create_inmemory(),
            config: Config::new(),
            sys: Arc::new(MockSys {}),
        }
    }

    pub async fn create() -> Self {
        let repos = Repos::create_mongodb()
            .await
            .expect("Mongo db creds must be set and valid");
        Self {
            repos,
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }
}
