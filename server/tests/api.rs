mod helpers;

use helpers::sdk::NettuSDK;
use helpers::setup::spawn_app;

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let (_, sdk) = spawn_app().await;
    assert!(sdk.check_health().await.is_ok());
}

#[actix_web::main]
#[test]
async fn test_create_account() {
    let (app, sdk) = spawn_app().await;
    assert!(sdk
        .create_account(&app.config.create_account_secret_code)
        .await
        .is_ok());
}

#[actix_web::main]
#[test]
async fn test_get_account() {
    let (app, mut sdk) = spawn_app().await;
    let res = sdk
        .create_account(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    sdk.set_admin_key(res.secret_api_key);

    assert!(sdk.get_account().await.is_ok());
}
