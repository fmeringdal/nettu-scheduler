mod helpers;

use std::collections::HashMap;

use helpers::setup::spawn_app;
use nettu_scheduler_domain::PEMKey;
use nettu_scheduler_sdk::{
    AddServiceUserInput, CreateCalendarInput, CreateEventInput, CreateScheduleInput,
    CreateUserInput, DeleteCalendarInput, DeleteEventInput, GetCalendarEventsInput,
    GetCalendarInput, GetEventInput, GetEventsInstancesInput, GetSerivceBookingSlotsInput,
    KVMetadata, MetadataFindInput, NettuSDK, RemoveServiceUserInput, UpdateCalendarInput,
    UpdateEventInput, UpdateScheduleInput, UpdateServiceUserInput,
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
async fn test_crud_user() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let mut metadata = HashMap::new();
    metadata.insert("group_id".to_string(), "123".to_string());

    let res = admin_client
        .user
        .create(CreateUserInput {
            metadata: Some(metadata.clone()),
        })
        .await
        .expect("Expected to create user");
    assert_eq!(
        res.user.metadata.get("group_id").unwrap().clone(),
        "123".to_string()
    );

    let metadata = KVMetadata {
        key: "group_id".to_string(),
        value: "123".to_string(),
    };
    let meta_query = MetadataFindInput {
        limit: 100,
        skip: 0,
        metadata,
    };

    let users_by_meta = admin_client
        .user
        .get_by_meta(meta_query)
        .await
        .expect("To get users by meta");
    assert_eq!(users_by_meta.users.len(), 1);
    assert_eq!(users_by_meta.users[0].id, res.user.id);

    let get_user = admin_client
        .user
        .get(res.user.id.clone())
        .await
        .expect("To get user")
        .user;
    assert_eq!(get_user.id, res.user.id);

    let delete_user_res = admin_client
        .user
        .delete(res.user.id.clone())
        .await
        .expect("To delet euser")
        .user;
    assert_eq!(delete_user_res.id, res.user.id);

    // Now that user is deleted, get query should return 404 error
    assert!(admin_client.user.get(res.user.id.clone()).await.is_err());
}

#[actix_web::main]
#[test]
async fn test_crud_schedule() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let create_user_res = admin_client
        .user
        .create(CreateUserInput { metadata: None })
        .await
        .expect("Expected to create user");

    let schedule = admin_client
        .schedule
        .create(CreateScheduleInput {
            user_id: create_user_res.user.id.clone(),
            rules: None,
            timezone: "UTC".into(),
        })
        .await
        .expect("Expected to create schedule")
        .schedule;
    assert_eq!(schedule.user_id, create_user_res.user.id);
    assert_eq!(schedule.timezone, "UTC");
    assert_eq!(schedule.rules.len(), 5); // mon-fri

    let schedule = admin_client
        .schedule
        .update(UpdateScheduleInput {
            rules: Some(vec![]),
            timezone: Some("Europe/Oslo".into()),
            schedule_id: schedule.id.clone(),
        })
        .await
        .unwrap()
        .schedule;

    let get_schedule = admin_client
        .schedule
        .get(schedule.id.clone())
        .await
        .unwrap()
        .schedule;

    assert_eq!(get_schedule.rules.len(), 0);
    assert_eq!(get_schedule.timezone, "Europe/Oslo");

    assert!(admin_client
        .schedule
        .delete(schedule.id.clone())
        .await
        .is_ok());

    assert!(admin_client
        .schedule
        .get(schedule.id.clone())
        .await
        .is_err());
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
    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let create_user_res = admin_client
        .user
        .create(CreateUserInput { metadata: None })
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
    let user = admin_client
        .user
        .create(CreateUserInput { metadata: None })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: "UTC".into(),
            week_start: 0,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let calendar_get_res = admin_client
        .calendar
        .get(GetCalendarInput {
            calendar_id: calendar.id.clone(),
        })
        .await
        .unwrap()
        .calendar;

    assert_eq!(calendar_get_res.id, calendar.id);

    let events = admin_client
        .calendar
        .get_events(GetCalendarEventsInput {
            calendar_id: calendar.id.clone(),
            start_ts: 0,
            end_ts: 1000 * 60 * 60 * 24,
        })
        .await
        .unwrap();

    assert_eq!(events.events.len(), 0);

    let week_start = 2;
    let calendar_with_new_settings = admin_client
        .calendar
        .update(UpdateCalendarInput {
            calendar_id: calendar.id.clone(),
            timezone: None,
            week_start: Some(week_start.clone()),
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;
    assert_eq!(calendar_with_new_settings.settings.week_start, week_start);

    // Delete calendar
    assert!(admin_client
        .calendar
        .delete(DeleteCalendarInput {
            calendar_id: calendar.id.clone(),
        })
        .await
        .is_ok());

    // Get now returns 404
    assert!(admin_client
        .calendar
        .get(GetCalendarInput {
            calendar_id: calendar.id.clone(),
        })
        .await
        .is_err());
}

#[actix_web::main]
#[test]
async fn test_crud_events() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput { metadata: None })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: "UTC".into(),
            week_start: 0,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(
            user.id.clone(),
            CreateEventInput {
                calendar_id: calendar.id.clone(),
                busy: None,
                duration: 1000 * 60 * 60,
                reminder: None,
                recurrence: None,
                is_service: None,
                start_ts: 0,
                metadata: None,
            },
        )
        .await
        .unwrap()
        .event;
    assert_eq!(event.calendar_id, calendar.id);

    let event = admin_client
        .event
        .get(GetEventInput {
            event_id: event.id.clone(),
        })
        .await
        .unwrap()
        .event;
    assert_eq!(event.calendar_id, calendar.id);
    let event_instances = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_ts: 0,
            end_ts: 1000 * 60 * 60 * 24,
        })
        .await
        .unwrap()
        .instances;
    assert_eq!(event_instances.len(), 1);
    assert!(admin_client
        .event
        .update(UpdateEventInput {
            event_id: event.id.clone(),
            exdates: Some(vec![0]),
            busy: None,
            duration: None,
            reminder: None,
            rrule_options: None,
            is_service: None,
            start_ts: None,
            metadata: None,
        })
        .await
        .is_ok());
    let event_instances = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_ts: 0,
            end_ts: 1000 * 60 * 60 * 24,
        })
        .await
        .unwrap()
        .instances;
    assert_eq!(event_instances.len(), 0);

    let event = admin_client
        .event
        .delete(DeleteEventInput {
            event_id: event.id.clone(),
        })
        .await
        .unwrap()
        .event;
    assert_eq!(event.calendar_id, calendar.id);

    assert!(admin_client
        .event
        .get(GetEventInput {
            event_id: event.id.clone(),
        })
        .await
        .is_err())
}

#[actix_web::main]
#[test]
async fn test_crud_service() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NettuSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput { metadata: None })
        .await
        .unwrap()
        .user;

    let service = admin_client.service.create().await.unwrap().service;

    let service = admin_client
        .service
        .add_user(AddServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            availibility: None,
            buffer: None,
            busy: None,
            closest_booking_time: None,
            furthest_booking_time: None,
        })
        .await
        .unwrap()
        .service;

    assert_eq!(service.users.len(), 1);
    let new_closest_booking_time = service.users[0].closest_booking_time + 1000 * 60 * 60;
    let service = admin_client
        .service
        .update_user(UpdateServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            availibility: None,
            buffer: None,
            busy: None,
            closest_booking_time: Some(new_closest_booking_time),
            furthest_booking_time: None,
        })
        .await
        .unwrap()
        .service;

    assert_eq!(
        service.users[0].closest_booking_time,
        new_closest_booking_time
    );
    let service = admin_client
        .service
        .remove_user(RemoveServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
        })
        .await
        .unwrap()
        .service;
    assert!(service.users.is_empty());

    let booking_slots = admin_client
        .service
        .bookingslots(GetSerivceBookingSlotsInput {
            date: "2020-1-1".to_string(),
            duration: 1000 * 60 * 30,
            iana_tz: Some("UTC".to_string()),
            interval: 1000 * 60 * 15,
            service_id: service.id.clone(),
        })
        .await
        .unwrap()
        .booking_slots;
    assert!(booking_slots.is_empty());

    // Delete service
    assert!(admin_client
        .service
        .delete(service.id.clone())
        .await
        .is_ok());

    // Get now returns 404
    assert!(admin_client.service.get(service.id.clone()).await.is_err());
}
