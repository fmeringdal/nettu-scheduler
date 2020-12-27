use crate::{
    calendar::repo::{CalendarRepo, ICalendarRepo},
    event::repo::{EventRepo, IEventRepo},
};
use actix_web::web;
use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;
type DataContext = web::Data<Arc<Context>>;

pub struct Repos {
    pub event_repo: Arc<dyn IEventRepo>,
    pub calendar_repo: Arc<dyn ICalendarRepo>,
}

impl Repos {
    pub async fn create() -> Result<Self, Box<dyn std::error::Error>> {
        println!("create");
        let client_options =
            ClientOptions::parse(&std::env::var("MONGODB_CONNECTION_STRING").unwrap()).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(&std::env::var("MONGODB_NAME").unwrap());

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
        })
    }
}

pub struct Context {
    pub repos: Repos,
}

impl Context {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let repos = Repos::create().await?;

        Ok(Self { repos })
    }
}
