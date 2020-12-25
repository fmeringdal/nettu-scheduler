use crate::db::EventRepo;
use crate::event::{CalendarEvent, RRuleOptions};
use actix_web::{post, web};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

type DataContext = web::Data<Arc<Context>>;

#[derive(Serialize, Deserialize)]
pub struct CreateEventReq {
    start_ts: i64,
    duration: i64,
    rrule_options: Option<RRuleOptions>,
}

#[post("/events")]
pub async fn post_event(data: DataContext, event: web::Json<CreateEventReq>) -> &'static str {
    let mut e = CalendarEvent {
        start_ts: event.start_ts,
        duration: event.duration,
        recurrence: None,
        end_ts: None,
        exdates: vec![],
        calendar_id: String::from("1"),
        user_id: String::from("2"),
    };
    if let Some(rrule_opts) = event.rrule_options.clone() {
        e.set_reccurrence(rrule_opts);
    };
    data.repos.event_repo.insert(&e).await;
    "Hello, from create event we are up and running!\r\n"
}

pub struct Repos {
    pub event_repo: EventRepo,
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
            event_repo: EventRepo::new(&db),
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
