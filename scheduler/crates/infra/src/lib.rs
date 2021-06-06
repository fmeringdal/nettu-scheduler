mod config;
mod repos;
mod system;

pub use config::Config;
use repos::Repos;
pub use repos::{KVMetadata, MetadataFindQuery};
use std::sync::Arc;
pub use system::ISys;
use system::RealSys;
use tracing::{info, warn};

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
    fn create_inmemory() -> Self {
        Self {
            repos: Repos::create_inmemory(),
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }

    async fn create(params: ContextParams) -> Self {
        let repos = Repos::create_postgres(&params.postgres_connection_string)
            .await
            .expect("Mongo db credentials must be set and valid");
        Self {
            repos,
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        }
    }
}

/// Will setup the correct Infra Context given the environment
pub async fn setup_context() -> NettuContext {
    const PSQL_CONNECTION_STRING: &str = "POSTGRES_CONNECTION_STRING";

    let psql_connection_string = std::env::var(PSQL_CONNECTION_STRING);

    let args: Vec<_> = std::env::args().collect();

    // cargo run inmemory
    let inmemory_arg_set = args.len() > 1 && args[1].eq("inmemory");
    if inmemory_arg_set {
        info!("Inmemory argument provided. Going to use inmemory infra.");
        return NettuContext::create_inmemory();
    }

    if psql_connection_string.is_ok() {
        info!(
            "{} env var was provided. Going to use postgres.",
            PSQL_CONNECTION_STRING
        );
        NettuContext::create(ContextParams {
            postgres_connection_string: String::from("postgresql://localhost:5432/tester"),
        })
        .await
    } else {
        warn!(
            "{} env var was not provided. Going to use inmemory infra. This should only be used during testing!",
            PSQL_CONNECTION_STRING
        );
        NettuContext::create_inmemory()
    }
}
