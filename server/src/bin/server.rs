use nettu_scheduler::{configure_server_app, Config, Context, Repos};

use actix_web::{get, middleware, App, HttpServer};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().collect();
    let repos = if args.last() == Some(&String::from("inmemory")) {
        Repos::create_inmemory()
    } else {
        Repos::create_mongodb()
            .await
            .expect("Mongo db creds must be set and valid")
    };

    HttpServer::new(move || {
        let ctx = Context {
            repos: repos.clone(),
            config: Config::new(),
        };

        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .data(ctx)
            .configure(|cfg| configure_server_app(cfg))
    })
    .bind("0.0.0.0:5000")?
    .workers(4)
    .run()
    .await
}
