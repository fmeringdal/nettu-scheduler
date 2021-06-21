mod helpers;

use chrono::{Duration, Utc};
use helpers::setup::spawn_app;
use helpers::utils::format_datetime;
use nettu_scheduler_domain::{BusyCalendar, ServiceMultiPersonOptions, TimePlan, ID};
use nettu_scheduler_sdk::{
    AddServiceUserInput, Calendar, CreateBookingIntendInput, CreateCalendarInput, CreateEventInput,
    CreateScheduleInput, CreateServiceInput, CreateUserInput, GetServiceBookingSlotsInput,
    NettuSDK, User,
};

fn assert_equal_user_list(users1: &Vec<User>, users2: &Vec<User>) {
    assert_eq!(users1.len(), users2.len());
    let mut users1 = users1.clone();
    users1.sort_by_key(|u| u.id.to_string());
    let mut users2 = users2.clone();
    users2.sort_by_key(|u| u.id.to_string());
    for (user1, user2) in users1.iter().zip(users2) {
        assert_eq!(user1.id, user2.id);
    }
}

async fn create_default_service_host(admin_client: &NettuSDK, service_id: &ID) -> (User, Calendar) {
    let input = CreateUserInput { metadata: None };
    let host = admin_client
        .user
        .create(input)
        .await
        .expect("To create user")
        .user;

    let input = CreateScheduleInput {
        metadata: None,
        rules: None,
        timezone: "UTC".to_string(),
        user_id: host.id.clone(),
    };
    let schedule = admin_client
        .schedule
        .create(input)
        .await
        .expect("To create schedule")
        .schedule;
    let input = CreateCalendarInput {
        metadata: None,
        synced: None,
        timezone: "UTC".to_string(),
        user_id: host.id.clone(),
        week_start: 0,
    };
    let busy_calendar = admin_client
        .calendar
        .create(input)
        .await
        .expect("To create calendar")
        .calendar;

    let input = AddServiceUserInput {
        availability: Some(TimePlan::Schedule(schedule.id.clone())),
        buffer_after: None,
        buffer_before: None,
        busy: Some(vec![BusyCalendar::Nettu(busy_calendar.id.clone())]),
        closest_booking_time: None,
        furthest_booking_time: None,
        service_id: service_id.clone(),
        user_id: host.id.clone(),
    };
    admin_client
        .service
        .add_user(input)
        .await
        .expect("To add host to service");
    (host, busy_calendar)
}

#[actix_web::main]
#[test]
async fn test_collective_team_scheduling() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let users_count_list: Vec<usize> = vec![0, 1, 5, 10];
    for users_count in users_count_list {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::Collective),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

        let mut hosts_with_calendar = vec![];
        let mut hosts = vec![];
        for _ in 0..users_count {
            let host = create_default_service_host(&admin_client, &service.id).await;
            hosts.push(host.0.clone());
            hosts_with_calendar.push(host);
        }

        let tomorrow = Utc::now() + Duration::days(1);
        let next_week = tomorrow + Duration::days(7);
        let duration = 1000 * 60 * 30;
        let interval = 1000 * 60 * 30;
        let input = GetServiceBookingSlotsInput {
            duration,
            interval,
            service_id: service.id.clone(),
            iana_tz: Some("UTC".into()),
            end_date: format_datetime(&next_week),
            start_date: format_datetime(&tomorrow),
        };
        let bookingslots = admin_client
            .service
            .bookingslots(input)
            .await
            .expect("To get bookingslots")
            .dates;
        if users_count == 0 {
            assert!(bookingslots.is_empty());
            continue;
        }
        let available_slot = bookingslots[0].slots[0].start;

        let input = CreateBookingIntendInput {
            service_id: service.id.clone(),
            host_user_ids: None,
            timestamp: available_slot,
            duration,
            interval,
        };
        let booking_intend = admin_client
            .service
            .create_booking_intend(input)
            .await
            .expect("To create booking intend");
        assert_equal_user_list(&booking_intend.selected_hosts, &hosts);
        assert_eq!(booking_intend.create_event_for_hosts, true);

        for (host, calendar) in hosts_with_calendar {
            let service_event = CreateEventInput {
                busy: Some(true),
                calendar_id: calendar.id.clone(),
                duration,
                metadata: None,
                recurrence: None,
                reminder: None,
                service_id: Some(service.id.clone()),
                start_ts: available_slot,
            };
            admin_client
                .event
                .create(host.id.clone(), service_event)
                .await
                .expect("To create service event");
        }

        // Now there are no more spots available so booking intend should fail
        let input = CreateBookingIntendInput {
            service_id: service.id.clone(),
            host_user_ids: None,
            timestamp: available_slot,
            duration,
            interval,
        };
        admin_client
            .service
            .create_booking_intend(input)
            .await
            .expect_err("Expected timestamp to now be full booked");

        // And bookingslots query also no longer shows that time
        let input = GetServiceBookingSlotsInput {
            duration,
            interval,
            service_id: service.id.clone(),
            iana_tz: Some("UTC".into()),
            end_date: format_datetime(&next_week),
            start_date: format_datetime(&tomorrow),
        };
        let bookingslots = admin_client
            .service
            .bookingslots(input)
            .await
            .expect("To get bookingslots")
            .dates;
        let available_slot_after_first_full_booking = bookingslots[0].slots[0].start;
        assert_ne!(available_slot, available_slot_after_first_full_booking);
    }
}

#[actix_web::main]
#[test]
async fn test_collective_team_scheduling_is_collective() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let input = CreateServiceInput {
        metadata: None,
        multi_person: Some(ServiceMultiPersonOptions::Collective),
    };
    let service = admin_client
        .service
        .create(input)
        .await
        .expect("To create service")
        .service;

    let input = CreateUserInput { metadata: None };
    let host1 = admin_client
        .user
        .create(input)
        .await
        .expect("To create user")
        .user;
    let input = CreateUserInput { metadata: None };
    let host2 = admin_client
        .user
        .create(input)
        .await
        .expect("To create user")
        .user;

    let input = CreateScheduleInput {
        metadata: None,
        rules: None,
        timezone: "UTC".to_string(),
        user_id: host1.id.clone(),
    };
    let schedule = admin_client
        .schedule
        .create(input)
        .await
        .expect("To create schedule")
        .schedule;

    // Add host 1
    let input = AddServiceUserInput {
        availability: Some(TimePlan::Schedule(schedule.id.clone())),
        buffer_after: None,
        buffer_before: None,
        busy: None,
        closest_booking_time: None,
        furthest_booking_time: None,
        service_id: service.id.clone(),
        user_id: host1.id.clone(),
    };
    admin_client
        .service
        .add_user(input)
        .await
        .expect("To add host to service");

    // Add host 2
    let input = AddServiceUserInput {
        availability: Some(TimePlan::Empty),
        buffer_after: None,
        buffer_before: None,
        busy: None,
        closest_booking_time: None,
        furthest_booking_time: None,
        service_id: service.id.clone(),
        user_id: host2.id.clone(),
    };
    admin_client
        .service
        .add_user(input)
        .await
        .expect("To add host to service");

    let tomorrow = Utc::now() + Duration::days(1);
    let next_week = tomorrow + Duration::days(7);
    let duration = 1000 * 60 * 30;
    let interval = 1000 * 60 * 30;
    let input = GetServiceBookingSlotsInput {
        duration,
        interval,
        service_id: service.id.clone(),
        iana_tz: Some("UTC".into()),
        end_date: format_datetime(&next_week),
        start_date: format_datetime(&tomorrow),
    };
    let bookingslots = admin_client
        .service
        .bookingslots(input)
        .await
        .expect("To get bookingslots")
        .dates;

    // Host 1 is available but host 2 is neveer available
    assert!(bookingslots.is_empty());
}