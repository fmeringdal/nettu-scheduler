mod config;
mod repos;
mod system;

pub use config::Config;
pub use mongodb::bson::oid::ObjectId;
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
        info!("Inmemory argument provided. Going to use inmemory infra.");
        return NettuContext::create_inmemory();
    }

    if mongodb_conncetion_string.is_ok() && mongodb_db_name.is_ok() {
        info!(
            "{} and {} env vars was provided. Going to use mongodb.",
            MONGODB_CONNECTION_STRING, MONGODB_NAME
        );
        NettuContext::create(ContextParams {
            mongodb: (mongodb_conncetion_string.unwrap(), mongodb_db_name.unwrap()),
        })
        .await
    } else {
        warn!(
            "{} and {} env vars was not provided. Going to use inmemory infra. This should only be used during testing!",
            MONGODB_CONNECTION_STRING, MONGODB_NAME
        );
        NettuContext::create_inmemory()
    }
}
