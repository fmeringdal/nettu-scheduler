use crate::db::{EventRepo, IEventRepo};
use crate::event::{CalendarEvent, RRuleOptions};
use actix_web::Responder;
use actix_web::{post, web, App};
use async_trait::async_trait;
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
type DataContext = web::Data<Arc<Context>>;

#[derive(Serialize, Deserialize)]
pub struct CreateEventReq {
    start_ts: i64,
    duration: i64,
    rrule_options: Option<RRuleOptions>,
}

pub fn configure_routes(cfg: &mut web::ServiceConfig, event_repo: Arc<dyn IEventRepo>) {
    let create_event_usecase = CreateEventUseCase {
        event_repo: Arc::clone(&event_repo),
    };
    cfg.app_data(web::Data::new(create_event_usecase));
    cfg.route("/events", web::post().to(create_event_controller));
}

async fn create_event_controller(
    create_event_usecase: web::Data<CreateEventUseCase>,
    req: web::Json<CreateEventReq>,
) -> impl Responder {
    let res = create_event_usecase.execute(req.0).await;
    "Hello, from create event we are up and running!\r\n"
}

pub struct Repos {
    pub event_repo: Arc<dyn IEventRepo>,
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

#[async_trait(?Send)]
trait UseCase<IReq, IRes> {
    async fn execute(&self, req: IReq) -> IRes;
}

struct CreateEventUseCase {
    pub event_repo: Arc<dyn IEventRepo>,
}

#[async_trait(?Send)]
impl UseCase<CreateEventReq, Result<(), Box<dyn Error>>> for CreateEventUseCase {
    async fn execute(&self, event: CreateEventReq) -> Result<(), Box<dyn Error>> {
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
        self.event_repo.insert(&e).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MockEventRepo {}

    #[async_trait]
    impl IEventRepo for MockEventRepo {
        async fn insert(&self, e: &CalendarEvent) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        async fn find(&self, event_id: &str) -> Option<CalendarEvent> {
            None
        }
    }

    #[actix_web::main]
    #[test]
    async fn create_event_use_case_test() {
        let use_case = CreateEventUseCase {
            event_repo: Arc::new(MockEventRepo {}),
        };
        let res = use_case
            .execute(CreateEventReq {
                start_ts: 500,
                duration: 800,
                rrule_options: None,
            })
            .await;
        assert!(res.is_ok());
    }
}
