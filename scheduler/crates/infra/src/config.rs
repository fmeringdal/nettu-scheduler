use nettu_scheduler_utils::create_random_secret;
use tracing::{info, log::warn};

#[derive(Debug, Clone)]
pub struct Config {
    /// Secret code used to create new `Account`s
    pub create_account_secret_code: String,
    /// Port for the application to run on
    pub port: usize,
    /// Maximum allowed duration in millis for querying event instances.
    /// This is used to avoid having clients ask for `CalendarEvents` in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    pub event_instances_query_duration_limit: i64,
    /// Maximum allowed duration in millis for querying booking slots
    /// This is used to avoid having clients ask for `BookingSlot`s in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    pub booking_slots_query_duration_limit: i64,
}

impl Config {
    pub fn new() -> Self {
        let create_account_secret_code = match std::env::var("CREATE_ACCOUNT_SECRET_CODE") {
            Ok(code) => code,
            Err(_) => {
                info!("Did not find CREATE_ACCOUNT_SECRET_CODE environment variable. Going to create one.");
                let code = create_random_secret(16);
                info!(
                    "Secret code for creating accounts was generated and set to: {}",
                    code
                );
                code
            }
        };
        let default_port = "5000";
        let port = std::env::var("PORT").unwrap_or(default_port.into());
        let port = match port.parse::<usize>() {
            Ok(port) => port,
            Err(_) => {
                warn!(
                    "The given PORT: {} is not valid, falling back to the default port: {}.",
                    port, default_port
                );
                default_port.parse::<usize>().unwrap()
            }
        };
        Self {
            create_account_secret_code,
            port,
            event_instances_query_duration_limit: 1000 * 60 * 60 * 24 * 62, // 62 days
            booking_slots_query_duration_limit: 1000 * 60 * 60 * 24 * 7,    // 7 days
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
