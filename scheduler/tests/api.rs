mod helpers;

use helpers::setup::spawn_app;
use nettu_scheduler_domain::{PEMKey, Weekday};
use nettu_scheduler_sdk::{
    AddServiceUserInput, CreateCalendarInput, CreateEventInput, CreateScheduleInput,
    CreateServiceInput, CreateUserInput, GetCalendarEventsInput, GetEventsInstancesInput,
    GetServiceBookingSlotsInput, GetUserFreeBusyInput, KVMetadata, MetadataFindInput, NettuSDK,
    RemoveServiceUserInput, UpdateCalendarInput, UpdateEventInput, UpdateScheduleInput,
    UpdateServiceUserInput,
};
use std::collections::HashMap;

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
            metadata: Some(metadata.into()),
        })
        .await
        .expect("Expected to create user");
    assert_eq!(
        res.user.metadata.inner.get("group_id").unwrap().clone(),
        "123".to_string()
    );

    let free_busy_req = GetUserFreeBusyInput {
        start_ts: 0,
        end_ts: 10,
        calendar_ids: None,
        user_id: res.user.id.clone(),
    };
    let free_busy_res = admin_client.user.free_busy(free_busy_req).await;
    assert!(free_busy_res.is_ok());

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
        .expect("To delete user")
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
            timezone: chrono_tz::UTC,
            metadata: None,
        })
        .await
        .expect("Expected to create schedule")
        .schedule;
    assert_eq!(schedule.user_id, create_user_res.user.id);
    assert_eq!(schedule.timezone, chrono_tz::UTC);
    assert_eq!(schedule.rules.len(), 7);

    let schedule = admin_client
        .schedule
        .update(UpdateScheduleInput {
            rules: Some(Vec::new()),
            timezone: Some(chrono_tz::Europe::Oslo),
            schedule_id: schedule.id.clone(),
            metadata: None,
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
    assert_eq!(get_schedule.timezone, chrono_tz::Europe::Oslo);

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
            timezone: chrono_tz::UTC,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let calendar_get_res = admin_client
        .calendar
        .get(calendar.id.clone())
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

    let week_start = Weekday::Wed;
    let calendar_with_new_settings = admin_client
        .calendar
        .update(UpdateCalendarInput {
            calendar_id: calendar.id.clone(),
            timezone: None,
            week_start: Some(week_start),
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;
    assert_eq!(calendar_with_new_settings.settings.week_start, week_start);

    // Delete calendar
    assert!(admin_client
        .calendar
        .delete(calendar.id.clone())
        .await
        .is_ok());

    // Get now returns 404
    assert!(admin_client
        .calendar
        .get(calendar.id.clone())
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
            timezone: chrono_tz::UTC,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: None,
            service_id: None,
            start_ts: 0,
            metadata: None,
        })
        .await
        .unwrap()
        .event;
    assert_eq!(event.calendar_id, calendar.id);

    let event = admin_client
        .event
        .get(event.id.clone())
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
            reminders: None,
            rrule_options: None,
            service_id: None,
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
        .delete(event.id.clone())
        .await
        .unwrap()
        .event;
    assert_eq!(event.calendar_id, calendar.id);

    assert!(admin_client.event.get(event.id.clone()).await.is_err())
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

    let create_service_input = CreateServiceInput {
        metadata: None,
        multi_person: None,
    };
    let service = admin_client
        .service
        .create(create_service_input)
        .await
        .unwrap()
        .service;

    let add_user_res = admin_client
        .service
        .add_user(AddServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            availability: None,
            buffer_after: None,
            buffer_before: None,
            closest_booking_time: None,
            furthest_booking_time: None,
        })
        .await;
    assert!(add_user_res.is_ok());
    let added_service_resource = add_user_res.unwrap();
    assert_eq!(added_service_resource.user_id, user.id.clone());
    assert_eq!(added_service_resource.service_id, service.id.clone());

    let service = admin_client.service.get(service.id.clone()).await.unwrap();

    assert_eq!(service.users.len(), 1);
    let new_closest_booking_time = service.users[0].closest_booking_time + 1000 * 60 * 60;

    let service_resource = admin_client
        .service
        .update_user(UpdateServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
            availability: None,
            buffer_after: None,
            buffer_before: None,
            closest_booking_time: Some(new_closest_booking_time),
            furthest_booking_time: None,
        })
        .await
        .unwrap();

    assert_eq!(
        service_resource.closest_booking_time,
        new_closest_booking_time
    );
    let remove_user_res = admin_client
        .service
        .remove_user(RemoveServiceUserInput {
            service_id: service.id.clone(),
            user_id: user.id.clone(),
        })
        .await;
    assert!(remove_user_res.is_ok());

    let service = admin_client.service.get(service.id.clone()).await.unwrap();
    assert!(service.users.is_empty());

    let booking_slots = admin_client
        .service
        .bookingslots(GetServiceBookingSlotsInput {
            start_date: "2030-1-1".to_string(),
            end_date: "2030-1-2".to_string(),
            duration: 1000 * 60 * 30,
            timezone: Some(chrono_tz::UTC),
            interval: 1000 * 60 * 15,
            host_user_ids: None,
            service_id: service.id.clone(),
        })
        .await
        .unwrap()
        .dates;
    assert!(booking_slots.is_empty());

    // About 100 days timespan
    let booking_slots = admin_client
        .service
        .bookingslots(GetServiceBookingSlotsInput {
            start_date: "2030-1-1".to_string(),
            end_date: "2030-4-1".to_string(),
            duration: 1000 * 60 * 30,
            timezone: Some(chrono_tz::UTC),
            interval: 1000 * 60 * 15,
            host_user_ids: None,
            service_id: service.id.clone(),
        })
        .await
        .unwrap()
        .dates;
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
