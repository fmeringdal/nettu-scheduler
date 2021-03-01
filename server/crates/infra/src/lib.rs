mod repos;
mod system;

use nettu_scheduler_utils::create_random_secret;
use repos::Repos;
use std::sync::Arc;
pub use system::ISys;
use system::RealSys;

pub use mongodb::bson::oid::ObjectId;

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
