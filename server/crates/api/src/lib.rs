mod account;
mod calendar;
mod error;
mod event;
mod job_schedulers;
mod schedule;
mod service;
mod shared;
mod status;
mod user;

use actix_web::{dev::Server, middleware, web, App, HttpServer};
use job_schedulers::{start_reminders_expansion_job_scheduler, start_send_reminders_job};
use nettu_scheduler_infra::NettuContext;

pub fn configure_server_api(cfg: &mut web::ServiceConfig) {
    account::api::configure_routes(cfg);
    calendar::api::configure_routes(cfg);
    event::api::configure_routes(cfg);
    service::api::configure_routes(cfg);
    schedule::api::configure_routes(cfg);
    status::api::configure_routes(cfg);
    user::api::configure_routes(cfg);
}

pub struct Application {
    server: Server,
}

impl Application {
    pub async fn new(context: NettuContext) -> Result<Self, std::io::Error> {
        let server = Application::configure_server(context.clone()).await?;
        Application::start_job_schedulers(context).await;

        Ok(Self { server })
    }

    async fn start_job_schedulers(context: NettuContext) {
        start_send_reminders_job(context.clone()).await;
        start_reminders_expansion_job_scheduler(context.clone()).await;
    }

    async fn configure_server(context: NettuContext) -> Result<Server, std::io::Error> {
        let port = context.config.port;

        let server = HttpServer::new(move || {
            let ctx = context.clone();

            App::new()
                .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .data(ctx)
                .configure(|cfg| configure_server_api(cfg))
        })
        .bind(format!("0.0.0.0:{}", port))?
        .workers(4)
        .run();

        Ok(server)
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
