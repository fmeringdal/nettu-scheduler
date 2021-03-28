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

use actix_cors::Cors;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use job_schedulers::{start_reminders_expansion_job_scheduler, start_send_reminders_job};
use nettu_scheduler_infra::NettuContext;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn configure_server_api(cfg: &mut web::ServiceConfig) {
    account::configure_routes(cfg);
    calendar::configure_routes(cfg);
    event::configure_routes(cfg);
    schedule::configure_routes(cfg);
    service::configure_routes(cfg);
    status::configure_routes(cfg);
    user::configure_routes(cfg);
}

pub struct Application {
    server: Server,
    port: u16,
}

impl Application {
    pub async fn new(context: NettuContext) -> Result<Self, std::io::Error> {
        let (server, port) = Application::configure_server(context.clone()).await?;
        Application::start_job_schedulers(context);

        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn start_job_schedulers(context: NettuContext) {
        start_send_reminders_job(context.clone());
        start_reminders_expansion_job_scheduler(context);
    }

    async fn configure_server(context: NettuContext) -> Result<(Server, u16), std::io::Error> {
        let port = context.config.port;
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();

        let server = HttpServer::new(move || {
            let ctx = context.clone();

            App::new()
                .wrap(Cors::permissive())
                .wrap(middleware::Compress::default())
                .wrap(TracingLogger)
                .data(ctx)
                .service(web::scope("/api/v1").configure(|cfg| configure_server_api(cfg)))
            // .configure(|cfg| configure_server_api(cfg))
        })
        .listen(listener)?
        .workers(4)
        .run();

        Ok((server, port))
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
