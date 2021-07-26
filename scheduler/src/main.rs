mod telemetry;

use nettu_scheduler_api::Application;
use nettu_scheduler_infra::setup_context;
use telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    openssl_probe::init_ssl_cert_env_vars();

    let subscriber = get_subscriber("nettu_scheduler_server".into(), "info".into());
    init_subscriber(subscriber);

    let context = setup_context().await;

    let app = Application::new(context).await?;
    app.start().await
}
