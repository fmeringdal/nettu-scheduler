mod account;
mod calendar;
mod event;
mod logger;
mod schedule;
mod service;
mod shared;
mod user;

use account::{AccountRepo, IAccountRepo, InMemoryAccountRepo};
use calendar::{CalendarRepo, ICalendarRepo, InMemoryCalendarRepo};
use chrono::Utc;
use event::{
    EventRemindersExpansionsJobRepo, EventRepo, IEventRemindersExpansionJobsRepo, IEventRepo,
    IReminderRepo, InMemoryEventRemindersExpansionJobsRepo, InMemoryEventRepo,
    InMemoryReminderRepo, ReminderRepo,
};
use mongodb::{options::ClientOptions, Client};
use nettu_scheduler_utils::create_random_secret;
use schedule::{IScheduleRepo, InMemoryScheduleRepo, ScheduleRepo};
use service::{IServiceRepo, InMemoryServiceRepo, ServiceRepo};
use std::{env::var, sync::Arc};
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
            event_reminders_expansion_jobs_repo: Arc::new(EventRemindersExpansionsJobRepo::new(
                &db,
            )),
        })
    }

    pub fn create_inmemory() -> Self {
        Self {
            event_repo: Arc::new(InMemoryEventRepo::new()),
            calendar_repo: Arc::new(InMemoryCalendarRepo::new()),
            account_repo: Arc::new(InMemoryAccountRepo::new()),
            user_repo: Arc::new(InMemoryUserRepo::new()),
            service_repo: Arc::new(InMemoryServiceRepo::new()),
            schedule_repo: Arc::new(InMemoryScheduleRepo::new()),
            reminder_repo: Arc::new(InMemoryReminderRepo::new()),
            event_reminders_expansion_jobs_repo: Arc::new(
                InMemoryEventRemindersExpansionJobsRepo::new(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub create_account_secret_code: String,
    pub port: usize,
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
        let port = std::env::var("PORT").unwrap_or("5000".into());
        let port = match port.parse::<usize>() {
            Ok(port) => port,
            Err(_) => {
                println!(
                    "The given PORT: {} is not valid, using the default port instead.",
                    port
                );
                5000
            }
        };
        Self {
            create_account_secret_code,
            port,
        }
    }
}

// Mocking out time so that it
// is possible to run tests depending
// that check the current timestamp
pub trait ISys: Send + Sync {
    /// The current timestamp in millis
    fn get_timestamp_millis(&self) -> i64;
}

pub struct RealSys {}
impl ISys for RealSys {
    fn get_timestamp_millis(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}

#[derive(Clone)]
pub struct NettuContext {
    pub repos: Repos,
    pub config: Config,
    pub sys: Arc<dyn ISys>,
}

struct ContextParams {
    // (connection_string, db_name)
    pub mongodb: (String, String),
}

impl NettuContext {
    fn create_inmemory() -> Self {
        Self {
            repos: Repos::create_inmemory(),
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }

    async fn create(params: ContextParams) -> Self {
        let repos = Repos::create_mongodb(&params.mongodb.0, &params.mongodb.1)
            .await
            .expect("Mongo db creds must be set and valid");
        Self {
            repos,
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }
}

/// Will setup the correct Infra Context given the environment
pub async fn setup_context() -> NettuContext {
    const MONGODB_CONNECTION_STRING: &str = "MONGODB_CONNECTION_STRNG";
    const MONGODB_NAME: &str = "MONGODB_NAME";

    let mongodb_conncetion_string = std::env::var(MONGODB_CONNECTION_STRING);
    let mongodb_db_name = std::env::var(MONGODB_NAME);

    let args: Vec<_> = std::env::args().collect();

    // cargo run inmemory
    let inmemory_arg_set = args.len() > 1 && args[1].eq("inmemory");
    if inmemory_arg_set {
        println!("Inmemory argument provided. Going to use inmemory infra.");
        return NettuContext::create_inmemory();
    }

    if mongodb_conncetion_string.is_ok() && mongodb_db_name.is_ok() {
        println!(
            "{} and {} env vars was provided. Going to use mongodb.",
            MONGODB_CONNECTION_STRING, MONGODB_NAME
        );
        NettuContext::create(ContextParams {
            mongodb: (mongodb_conncetion_string.unwrap(), mongodb_db_name.unwrap()),
        })
        .await
    } else {
        println!(
            "{} and {} env vars was not provided. Going to use inmemory infra.",
            MONGODB_CONNECTION_STRING, MONGODB_NAME
        );
        NettuContext::create_inmemory()
    }
}
