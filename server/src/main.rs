use env_logger::Env;
use nettu_scheduler_api::Application;
use nettu_scheduler_infra::setup_context;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let context = setup_context().await;

    let app = Application::new(context).await?;
    app.start().await
}
