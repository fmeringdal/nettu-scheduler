use crate::{
    account::repos::{AccountRepo, IAccountRepo, InMemoryAccountRepo},
    calendar::repos::{CalendarRepo, ICalendarRepo, InMemoryCalendarRepo},
    event::repos::{
        EventRepo, IEventRepo, IReminderRepo, InMemoryEventRepo, InMemoryReminderRepo, ReminderRepo,
    },
    schedule::{self, repos::IScheduleRepo},
    service::repos::{IServiceRepo, InMemoryServiceRepo, ServiceRepo},
    shared::utils::create_random_secret,
    user::repos::{IUserRepo, InMemoryUserRepo, UserRepo},
};
use actix_web::{
    dev::HttpResponseBuilder,
    http::{header, StatusCode},
    HttpResponse,
};
use chrono::prelude::*;
use mongodb::{options::ClientOptions, Client};
use schedule::repos::{InMemoryScheduleRepo, ScheduleRepo};
use std::sync::Arc;
use thiserror::Error;

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
        let client_options =
            ClientOptions::parse(&std::env::var("MONGODB_CONNECTION_STRING").expect("Expected MONGODB_CONNECTION_STRING env to be present")).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(&std::env::var("MONGODB_NAME").expect("Expected MONGODB_NAME env to be present"));

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

#[derive(Error, Debug)]
pub enum NettuError {
    #[error("data store disconnected")]
    InternalError,
    #[error("Invalid data provided: Error message: `{0}`")]
    BadClientData(String),
    #[error("There was a conflict with the request. Error message: `{0}`")]
    Conflict(String),
    #[error("Unauthorized request. Error message: `{0}`")]
    Unauthorized(String),
    #[error(
        "Unidentifiable client. Must include the `nettu-account` header. Error message: `{0}`"
    )]
    UnidentifiableClient(String),
    #[error("404 Not found. Error message: `{0}`")]
    NotFound(String),
}

impl actix_web::error::ResponseError for NettuError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            NettuError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            NettuError::BadClientData(_) => StatusCode::BAD_REQUEST,
            NettuError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            NettuError::Conflict(_) => StatusCode::CONFLICT,
            NettuError::NotFound(_) => StatusCode::NOT_FOUND,
            NettuError::UnidentifiableClient(_) => StatusCode::UNAUTHORIZED,
        }
    }
}
