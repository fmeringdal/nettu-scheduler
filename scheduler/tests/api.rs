mod helpers;

use helpers::setup::spawn_app;
use nettu_scheduler_domain::PEMKey;
use nettu_scheduler_sdk::{
    CreateCalendarInput, CreateScheduleInput, DeleteCalendarInput, GetCalendarEventsInput,
    GetCalendarInput, NettuSDK, UpdateCalendarSettingsInput,
};

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

#[actix_web::main]
#[test]
async fn test_crud_user() {
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
    let get_user_res = admin_client
        .user
        .get(create_user_res.user.id.clone())
        .await
        .expect("Expected to get user");
    assert_eq!(get_user_res.user.id, create_user_res.user.id);
    let delete_user_res = admin_client
        .user
        .delete(create_user_res.user.id.clone())
        .await
        .expect("Expected to delete user");
    assert_eq!(delete_user_res.user.id, create_user_res.user.id);

    // Get after deleted should be error
    let get_user_res = admin_client.user.get(create_user_res.user.id.clone()).await;
    assert!(get_user_res.is_err());
}

#[actix_web::main]
#[test]
async fn test_crud_account() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NettuSDK::new(address, res.secret_api_key);

    // Setting webhook url
    let webhook_url = "https://example.com";
    admin_client
        .account
        .create_webhook(webhook_url)
        .await
        .expect("Expected to create webhook");
    let account = admin_client.account.get().await.unwrap();
    assert_eq!(account.account.settings.webhook.unwrap().url, webhook_url);

    // Setting pub jwt key
    let key =
        String::from_utf8(std::fs::read("./crates/api/config/test_public_rsa_key.crt").unwrap())
            .unwrap();
    admin_client
        .account
        .set_account_pub_key(Some(key.clone()))
        .await
        .expect("Expected to set account jwt key");
    let account = admin_client.account.get().await.unwrap();
    assert_eq!(
        account.account.public_jwt_key,
        Some(PEMKey::new(key).unwrap())
    );

    // Removing pub jwt key
    admin_client
        .account
        .set_account_pub_key(None)
        .await
        .expect("Expected to remove account jwt key");
    let account = admin_client.account.get().await.unwrap();
    assert_eq!(account.account.public_jwt_key, None);

    // Removing webhook url
    admin_client
        .account
        .delete_webhook()
        .await
        .expect("Expected to delete account webhook");
    let account = admin_client.account.get().await.unwrap();
    assert!(account.account.settings.webhook.is_none());
}

#[actix_web::main]
#[test]
async fn test_crud_calendars() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let user = admin_client.user.create().await.unwrap().user;

    let calendar = admin_client
        .calendar
        .create(&CreateCalendarInput {
            user_id: user.id.to_string(),
            timezone: "UTC".into(),
            week_start: 0,
        })
        .await
        .unwrap()
        .calendar;

    let calendar_get_res = admin_client
        .calendar
        .get(&GetCalendarInput {
            calendar_id: calendar.id.to_string(),
        })
        .await
        .unwrap()
        .calendar;

    assert_eq!(calendar_get_res.id, calendar.id);

    let events = admin_client
        .calendar
        .get_events(&GetCalendarEventsInput {
            calendar_id: calendar.id.to_string(),
            start_ts: 0,
            end_ts: 1000 * 60 * 60 * 24,
        })
        .await
        .unwrap();

    assert_eq!(events.events.len(), 0);

    let week_start = 2;
    let calendar_with_new_settings = admin_client
        .calendar
        .update_settings(&UpdateCalendarSettingsInput {
            calendar_id: calendar.id.to_string(),
            timezone: None,
            week_start: Some(week_start.clone()),
        })
        .await
        .unwrap()
        .calendar;
    assert_eq!(calendar_with_new_settings.settings.week_start, week_start);

    // Delete calendar
    assert!(admin_client
        .calendar
        .delete(&DeleteCalendarInput {
            calendar_id: calendar.id.to_string(),
        })
        .await
        .is_ok());

    // Get now returns 404
    assert!(admin_client
        .calendar
        .get(&GetCalendarInput {
            calendar_id: calendar.id.to_string(),
        })
        .await
        .is_err());
}
