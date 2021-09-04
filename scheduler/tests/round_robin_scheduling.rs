mod helpers;

use chrono::{Duration, Utc, Weekday};
use helpers::setup::spawn_app;
use helpers::utils::{assert_equal_user_lists, format_datetime};
use nettu_scheduler_domain::{BusyCalendar, ServiceMultiPersonOptions, TimePlan, ID};
use nettu_scheduler_sdk::{
    AddBusyCalendar, AddServiceUserInput, Calendar, CreateBookingIntendInput, CreateCalendarInput,
    CreateEventInput, CreateScheduleInput, CreateServiceInput, CreateUserInput,
    GetServiceBookingSlotsInput, NettuSDK, RoundRobinAlgorithm, User,
};

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
        timezone: chrono_tz::UTC,
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
        timezone: chrono_tz::UTC,
        user_id: host.id.clone(),
        week_start: Weekday::Mon,
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
    let input = AddBusyCalendar {
        user_id: host.id.clone(),
        service_id: service_id.clone(),
        calendar: BusyCalendar::Nettu(busy_calendar.id.clone()),
    };
    admin_client
        .service
        .add_busy_calendar(input)
        .await
        .expect("To add busy calendar to service user");
    (host, busy_calendar)
}

#[actix_web::main]
#[test]
async fn test_round_robin_scheduling_simple_test() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let users_count_list: Vec<usize> = vec![0, 1, 5, 10];
    let round_robin_algos = vec![
        RoundRobinAlgorithm::Availability,
        RoundRobinAlgorithm::EqualDistribution,
    ];
    for users_count in users_count_list {
        for alg in round_robin_algos.clone() {
            let input = CreateServiceInput {
                metadata: None,
                multi_person: Some(ServiceMultiPersonOptions::RoundRobinAlgorithm(alg)),
            };
            let service = admin_client
                .service
                .create(input)
                .await
                .expect("To create service")
                .service;

            let mut hosts_with_calendar = Vec::new();
            let mut hosts = Vec::new();
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
                timezone: Some(chrono_tz::UTC),
                end_date: format_datetime(&next_week),
                start_date: format_datetime(&tomorrow),
                host_user_ids: None,
            };
            let bookingslots = admin_client
                .service
                .bookingslots(input.clone())
                .await
                .expect("To get bookingslots")
                .dates;
            if users_count == 0 {
                assert!(bookingslots.is_empty());
                continue;
            }
            let available_slot = bookingslots[0].slots[0].start;

            let mut booked_users = Vec::new();
            for _ in 0..users_count {
                let bookingslots = admin_client
                    .service
                    .bookingslots(input.clone())
                    .await
                    .expect("To get bookingslots")
                    .dates;
                assert_eq!(bookingslots[0].slots[0].start, available_slot);

                // Book the selected user
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

                // Only on select host for round robin
                assert_eq!(booking_intend.selected_hosts.len(), 1);
                booked_users.push(booking_intend.selected_hosts[0].clone());

                let (selected_host, busy_calendar) = hosts_with_calendar
                    .iter()
                    .find(|(h, _)| h.id == booking_intend.selected_hosts[0].id)
                    .expect("To find selected host");

                // Create service event
                let service_event = CreateEventInput {
                    user_id: selected_host.id.clone(),
                    busy: Some(true),
                    calendar_id: busy_calendar.id.clone(),
                    duration,
                    metadata: None,
                    recurrence: None,
                    reminders: Vec::new(),
                    service_id: Some(service.id.clone()),
                    start_ts: available_slot,
                };
                admin_client
                    .event
                    .create(service_event)
                    .await
                    .expect("To create service event");
            }
            // Make sure every host was booked once and only once
            assert_equal_user_lists(&booked_users, &hosts);

            // Now all hosts are taken for that timestamp
            // So the first available time is longer the same
            let bookingslots = admin_client
                .service
                .bookingslots(input.clone())
                .await
                .expect("To get bookingslots")
                .dates;
            assert_ne!(bookingslots[0].slots[0].start, available_slot);
        }
    }
}

#[actix_web::main]
#[test]
async fn test_round_robin_equal_distribution_scheduling() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    // Each test case is a list of upcoming service events for a host
    let test_cases: Vec<Vec<usize>> = vec![
        vec![3, 0, 1, 5],
        vec![0],
        Vec::new(),
        vec![2, 1, 1, 1, 1, 4],
        vec![1, 1, 0],
        vec![2, 7, 4],
        vec![1, 1, 1],
    ];

    for upcoming_service_events_per_host in test_cases {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::RoundRobinAlgorithm(
                RoundRobinAlgorithm::EqualDistribution,
            )),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

        let users_count = upcoming_service_events_per_host.len();
        let mut hosts = Vec::new();
        let mut hosts_with_calendars = Vec::new();
        for _ in 0..users_count {
            let host = create_default_service_host(&admin_client, &service.id).await;
            hosts.push(host.0.clone());
            hosts_with_calendars.push(host);
        }

        let duration = 1000 * 60 * 30;
        let interval = 1000 * 60 * 30;
        let tomorrow = Utc::now() + Duration::days(1);
        let next_week = tomorrow + Duration::days(7);
        let input = GetServiceBookingSlotsInput {
            duration,
            interval,
            service_id: service.id.clone(),
            timezone: Some(chrono_tz::UTC),
            end_date: format_datetime(&next_week),
            start_date: format_datetime(&tomorrow),
            host_user_ids: None,
        };
        let bookingslots = admin_client
            .service
            .bookingslots(input.clone())
            .await
            .expect("To get bookingslots")
            .dates;
        if users_count == 0 {
            assert!(bookingslots.is_empty());
            continue;
        }
        let available_slot = bookingslots[0].slots[0].start;
        let some_time_later = available_slot + 14 * 24 * 60 * 60 * 1000;

        // Create upcoming service_events
        for (upcoming_service_events, (host, busy_calendar)) in upcoming_service_events_per_host
            .iter()
            .zip(&hosts_with_calendars)
        {
            for _ in 0..*upcoming_service_events {
                // Create service event
                let service_event = CreateEventInput {
                    user_id: host.id.clone(),
                    busy: Some(true),
                    calendar_id: busy_calendar.id.clone(),
                    duration,
                    metadata: None,
                    recurrence: None,
                    reminders: Vec::new(),
                    service_id: Some(service.id.clone()),
                    start_ts: some_time_later,
                };
                admin_client
                    .event
                    .create(service_event)
                    .await
                    .expect("To create service event");
            }
        }

        let min_upcoming_events = upcoming_service_events_per_host.iter().min().unwrap();
        let mut hosts_with_min_upcoming_events = hosts
            .iter()
            .zip(&upcoming_service_events_per_host)
            .filter(|(_, count)| *count == min_upcoming_events)
            .map(|(h, _)| h.id.clone())
            .collect::<Vec<_>>();

        for _ in 0..hosts_with_min_upcoming_events.len() {
            // Book the selected user
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
            assert!(booking_intend.create_event_for_hosts);

            assert_eq!(booking_intend.selected_hosts.len(), 1);
            assert!(hosts_with_min_upcoming_events.contains(&booking_intend.selected_hosts[0].id));
            hosts_with_min_upcoming_events = hosts_with_min_upcoming_events
                .into_iter()
                .filter(|host_id| host_id != &booking_intend.selected_hosts[0].id)
                .collect();

            // Create service event for booking
            let (host, busy_calendar) = hosts_with_calendars
                .iter()
                .find(|(h, _)| h.id == booking_intend.selected_hosts[0].id)
                .expect("To find selected host");
            let service_event = CreateEventInput {
                user_id: host.id.clone(),
                busy: Some(true),
                calendar_id: busy_calendar.id.clone(),
                duration,
                metadata: None,
                recurrence: None,
                reminders: Vec::new(),
                service_id: Some(service.id.clone()),
                start_ts: available_slot,
            };
            admin_client
                .event
                .create(service_event)
                .await
                .expect("To create service event");
        }
    }
}

#[actix_web::main]
#[test]
async fn test_round_robin_availability_scheduling() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    // Each test case is a list of timestamps for the least recently assigned event for a host
    let test_cases: Vec<Vec<i64>> = vec![
        vec![3, 0, 1, 5],
        vec![0],
        Vec::new(),
        vec![2, 1, 1, 1, 1, 4],
        vec![1, 1, 0],
        vec![2, 7, 4],
        vec![1, 1, 1],
    ];

    for last_assigned_service_event_per_host in test_cases {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::RoundRobinAlgorithm(
                RoundRobinAlgorithm::Availability,
            )),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

        let users_count = last_assigned_service_event_per_host.len();
        let mut hosts = Vec::new();
        let mut hosts_with_calendars = Vec::new();
        for _ in 0..users_count {
            let host = create_default_service_host(&admin_client, &service.id).await;
            hosts.push(host.0.clone());
            hosts_with_calendars.push(host);
        }

        let duration = 1000 * 60 * 30;
        let interval = 1000 * 60 * 30;
        let tomorrow = Utc::now() + Duration::days(1);
        let next_week = tomorrow + Duration::days(7);
        let input = GetServiceBookingSlotsInput {
            duration,
            interval,
            service_id: service.id.clone(),
            timezone: Some(chrono_tz::UTC),
            end_date: format_datetime(&next_week),
            start_date: format_datetime(&tomorrow),
            host_user_ids: None,
        };
        let bookingslots = admin_client
            .service
            .bookingslots(input.clone())
            .await
            .expect("To get bookingslots")
            .dates;
        if users_count == 0 {
            assert!(bookingslots.is_empty());
            continue;
        }
        let available_slot = bookingslots[0].slots[0].start;

        // Create upcoming service_events
        for (last_assigned_service_event, (host, busy_calendar)) in
            last_assigned_service_event_per_host
                .iter()
                .zip(&hosts_with_calendars)
        {
            // Create service event
            let service_event = CreateEventInput {
                user_id: host.id.clone(),
                busy: Some(true),
                calendar_id: busy_calendar.id.clone(),
                duration,
                metadata: None,
                recurrence: None,
                reminders: Vec::new(),
                service_id: Some(service.id.clone()),
                start_ts: 0,
            };
            let event_id = admin_client
                .event
                .create(service_event)
                .await
                .expect("To create service event")
                .event
                .id;

            // Created field can only by updated through direct db queries
            let mut service_event = app
                .ctx
                .repos
                .events
                .find(&event_id)
                .await
                .expect("To find event");
            service_event.created = *last_assigned_service_event;
            app.ctx
                .repos
                .events
                .save(&service_event)
                .await
                .expect("To save event");
        }

        let last_assigned_event = last_assigned_service_event_per_host.iter().min().unwrap();
        let mut hosts_with_last_assigned_events = hosts
            .iter()
            .zip(&last_assigned_service_event_per_host)
            .filter(|(_, ts)| *ts == last_assigned_event)
            .map(|(h, _)| h.id.clone())
            .collect::<Vec<_>>();

        for _ in 0..hosts_with_last_assigned_events.len() {
            // Book the selected user
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
            assert!(booking_intend.create_event_for_hosts);

            assert_eq!(booking_intend.selected_hosts.len(), 1);
            assert!(hosts_with_last_assigned_events.contains(&booking_intend.selected_hosts[0].id));
            hosts_with_last_assigned_events = hosts_with_last_assigned_events
                .into_iter()
                .filter(|host_id| host_id != &booking_intend.selected_hosts[0].id)
                .collect();

            // Create service event for booking
            let (host, busy_calendar) = hosts_with_calendars
                .iter()
                .find(|(h, _)| h.id == booking_intend.selected_hosts[0].id)
                .expect("To find selected host");

            let service_event = CreateEventInput {
                user_id: host.id.clone(),
                busy: Some(true),
                calendar_id: busy_calendar.id.clone(),
                duration,
                metadata: None,
                recurrence: None,
                reminders: Vec::new(),
                service_id: Some(service.id.clone()),
                start_ts: available_slot,
            };
            admin_client
                .event
                .create(service_event)
                .await
                .expect("To create service event");
        }
    }
}
