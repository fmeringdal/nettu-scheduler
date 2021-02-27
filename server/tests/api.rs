mod helpers;

use helpers::spawn_app;
use nettu_scheduler_api::dev::account::CreateAccountResponse;

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let app = spawn_app().await;
    assert_eq!(app.check_health().await.status(), reqwest::StatusCode::OK);
}

#[actix_web::main]
#[test]
async fn test_create_account() {
    let app = spawn_app().await;
    assert!(app
        .create_account(&app.config.create_account_secret_code)
        .await
        .is_ok());
}

#[actix_web::main]
#[test]
async fn test_get_account() {
    let app = spawn_app().await;
    let res = app
        .create_account(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    assert!(app.get_account(&res.secret_api_key).await.is_ok());
}
