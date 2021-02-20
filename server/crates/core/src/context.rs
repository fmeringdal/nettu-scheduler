use crate::{
    account::repos::IAccountRepo,
    calendar::repos::ICalendarRepo,
    event::repos::{IEventRepo, IReminderRepo},
    schedule::repos::IScheduleRepo,
    service::repos::IServiceRepo,
    user::repos::IUserRepo,
};
use chrono::Utc;
use nettu_scheduler_utils::create_random_secret;
use std::sync::Arc;

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
