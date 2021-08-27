mod config;
mod repos;
mod services;
mod system;

pub use config::Config;
use repos::Repos;
pub use repos::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier, MetadataFindQuery};
pub use services::*;
use sqlx::migrate::MigrateError;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
pub use system::ISys;
use system::RealSys;

#[derive(Clone)]
pub struct NettuContext {
    pub repos: Repos,
    pub config: Config,
    pub sys: Arc<dyn ISys>,
}

struct ContextParams {
    pub postgres_connection_string: String,
}

impl NettuContext {
    async fn create(params: ContextParams) -> Self {
        let repos = Repos::create_postgres(&params.postgres_connection_string)
            .await
            .expect("Postgres credentials must be set and valid");
        Self {
            repos,
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }
}

/// Will setup the infrastructure context given the environment
pub async fn setup_context() -> NettuContext {
    NettuContext::create(ContextParams {
        postgres_connection_string: get_psql_connection_string(),
    })
    .await
}

fn get_psql_connection_string() -> String {
    const PSQL_CONNECTION_STRING: &str = "DATABASE_URL";

    std::env::var(PSQL_CONNECTION_STRING)
        .unwrap_or_else(|_| panic!("{} env var to be present.", PSQL_CONNECTION_STRING))
}

pub async fn run_migration() -> Result<(), MigrateError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&get_psql_connection_string())
        .await
        .expect("TO CONNECT TO POSTGRES");

    sqlx::migrate!().run(&pool).await
}
