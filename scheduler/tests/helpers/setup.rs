use std::time::Duration;

use actix_web::rt::time::{delay_until, Instant};
use nettu_scheduler_api::Application;
use nettu_scheduler_infra::{setup_context, Config};
use nettu_scheduler_sdk::NettuSDK;

pub struct TestApp {
    pub config: Config,
}

// Launch the application as a background task
pub async fn spawn_app() -> (TestApp, NettuSDK, String) {
    let mut ctx = setup_context().await;
    ctx.config.port = 0; // Random port
                         // ctx.config.create_account_secret_code = "123".into(); // Overriding create account secret

    let config = ctx.config.clone();
    let application = Application::new(ctx)
        .await
        .expect("Failed to build application.");

    let address = format!("http://localhost:{}", application.port());
    let _ = actix_web::rt::spawn(async move {
        application
            .start()
            .await
            .expect("Expected application to start");
    });

    // let instant = Instant::now() + Duration::from_millis(1000);
    // delay_until(instant).await;

    let app = TestApp { config };
    let sdk = NettuSDK::new(address.clone(), "");
    (app, sdk, address)
}
