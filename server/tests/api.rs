mod helpers;
use helpers::setup::spawn_app;
use nettu_scheduler_sdk::{CreateScheduleInput, NettuSDK};

#[actix_web::main]
#[test]
async fn test_status_ok() {
    let (_, sdk, _) = spawn_app().await;
    assert!(sdk.status.check_health().await.is_ok());
}

#[actix_web::main]
#[test]
async fn test_create_account() {
    let (app, sdk, _) = spawn_app().await;
    assert!(sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .is_ok());
}

#[actix_web::main]
#[test]
async fn test_get_account() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);
    assert!(admin_client.account.get().await.is_ok());
    assert!(sdk.account.get().await.is_err());
}

#[actix_web::main]
#[test]
async fn test_create_user() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let account = res.account;
    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let res = admin_client
        .user
        .create()
        .await
        .expect("Expected to create user");
    assert_eq!(res.user.account_id, account.id);
}

#[actix_web::main]
#[test]
async fn test_create_schedule() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let create_user_res = admin_client
        .user
        .create()
        .await
        .expect("Expected to create user");
    let res = admin_client
        .schedule
        .create(CreateScheduleInput {
            user_id: create_user_res.user.id.clone(),
            timezone: "UTC".into(),
        })
        .await
        .expect("Expected to create schedule");
    assert_eq!(res.schedule.user_id, create_user_res.user.id);
    assert_eq!(res.schedule.timezone, "UTC");
}
