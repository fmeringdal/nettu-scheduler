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

    let app = TestApp { config };
    let sdk = NettuSDK::new(address.clone(), "");
    (app, sdk, address)
}
