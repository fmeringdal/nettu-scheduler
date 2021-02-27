use crate::NettuSDK;

use nettu_scheduler_api::Application;
use nettu_scheduler_infra::{setup_context, Config};

pub struct TestApp {
    pub config: Config,
}

// Launch the application as a background task
pub async fn spawn_app() -> (TestApp, NettuSDK) {
    let mut ctx = setup_context().await;
    ctx.config.port = 0; // Random port
    ctx.config.create_account_secret_code = "123".into(); // Overriding create account secret

    let config = ctx.config.clone();
    let application = Application::new(ctx)
        .await
        .expect("Failed to build application.");

    let address = format!("http://localhost:{}", application.port());
    println!("{}", address);
    let _ = actix_web::rt::spawn(async move {
        application
            .start()
            .await
            .expect("Expected application to start");
    });

    let app = TestApp { config };
    let sdk = NettuSDK::new(address);
    (app, sdk)
}
