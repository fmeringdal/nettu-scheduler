mod config;
mod repos;
mod system;

pub use config::Config;
use repos::Repos;
pub use repos::{KVMetadata, MetadataFindQuery};
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
    const PSQL_CONNECTION_STRING: &str = "DATABASE_URL";

    let psql_connection_string = std::env::var(PSQL_CONNECTION_STRING).expect(&format!(
        "{} env var to be present.",
        PSQL_CONNECTION_STRING
    ));

    NettuContext::create(ContextParams {
        postgres_connection_string: psql_connection_string,
    })
    .await
}
