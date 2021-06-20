mod helpers;

use chrono::{Duration, Utc};
use helpers::setup::spawn_app;
use helpers::utils::format_datetime;
use nettu_scheduler_domain::{BusyCalendar, ServiceMultiPersonOptions, TimePlan, ID};
use nettu_scheduler_sdk::{
    AddServiceUserInput, Calendar, CreateBookingIntendInput, CreateCalendarInput, CreateEventInput,
    CreateScheduleInput, CreateServiceInput, CreateUserInput, GetEventInput,
    GetServiceBookingSlotsInput, NettuSDK, UpdateServiceInput, User,
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
async fn test_group_team_scheduling() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let users_count_list: Vec<usize> = vec![0, 1, 5, 10];
    let max_booking_spots_list = vec![0, 1, 2, 5, 10];
    for users_count in users_count_list {
        for max_booking_spots in max_booking_spots_list.clone() {
            let input = CreateServiceInput {
                metadata: None,
                multi_person: Some(ServiceMultiPersonOptions::Group(max_booking_spots)),
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
            if max_booking_spots == 0 || users_count == 0 {
                assert!(bookingslots.is_empty());
                continue;
            }
            let available_slot = bookingslots[0].slots[0].start;

            for _ in 0..max_booking_spots - 1 {
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
                assert_eq!(booking_intend.create_event_for_hosts, false);
            }
            // Last spot
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
}

#[actix_web::main]
#[test]
async fn test_group_team_scheduling_is_collective() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let max_booking_spots = 5;
    let input = CreateServiceInput {
        metadata: None,
        multi_person: Some(ServiceMultiPersonOptions::Group(max_booking_spots)),
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

#[actix_web::main]
#[test]
async fn test_group_team_scheduling_increase_max_count() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let test_set = vec![(5, 2), (1, 1), (1, 10), (10, 20)];
    for (max_booking_spots, booking_spots_inc) in test_set {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::Group(max_booking_spots)),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
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

        let available_slot = bookingslots[0].slots[0].start;

        for _ in 0..max_booking_spots - 1 {
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
            assert_eq!(
                booking_intend
                    .selected_hosts
                    .iter()
                    .map(|h| h.id.clone())
                    .collect::<Vec<_>>(),
                vec![host.id.clone()]
            );
            assert_eq!(booking_intend.create_event_for_hosts, false);
        }
        // Last spot
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
        assert_eq!(
            booking_intend
                .selected_hosts
                .iter()
                .map(|h| h.id.clone())
                .collect::<Vec<_>>(),
            vec![host.id.clone()]
        );
        assert_eq!(booking_intend.create_event_for_hosts, true);
        let service_event = CreateEventInput {
            busy: Some(true),
            calendar_id: busy_calendar.id.clone(),
            duration,
            metadata: None,
            recurrence: None,
            reminder: None,
            service_id: Some(service.id.clone()),
            start_ts: available_slot,
        };
        let service_event = admin_client
            .event
            .create(host.id.clone(), service_event)
            .await
            .expect("To create service event")
            .event;

        // Now there are no more spots available on that timestamp
        // But lets increase max count
        let input = UpdateServiceInput {
            metadata: None,
            service_id: service.id.clone(),
            multi_person: Some(ServiceMultiPersonOptions::Group(
                max_booking_spots + booking_spots_inc,
            )),
        };
        admin_client
            .service
            .update(input)
            .await
            .expect("To update service");

        // The current service event be deleted
        let input = GetEventInput {
            event_id: service_event.id.clone(),
        };
        assert!(admin_client.event.get(input).await.is_err());

        for _ in 0..booking_spots_inc - 1 {
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
            assert_eq!(
                booking_intend
                    .selected_hosts
                    .iter()
                    .map(|h| h.id.clone())
                    .collect::<Vec<_>>(),
                vec![host.id.clone()]
            );
            assert_eq!(booking_intend.create_event_for_hosts, false);
        }
        // And now the last spot
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
        assert_eq!(
            booking_intend
                .selected_hosts
                .iter()
                .map(|h| h.id.clone())
                .collect::<Vec<_>>(),
            vec![host.id.clone()]
        );
        assert_eq!(booking_intend.create_event_for_hosts, true);
    }

    // Increases from 0
    let test_set = vec![(0, 1), (0, 2), (0, 10)];
    for (max_booking_spots, booking_spots_inc) in test_set {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::Group(max_booking_spots)),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
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

        assert!(bookingslots.is_empty());

        // Now there are no more spots available on that timestamp
        // But lets increase max count
        let input = UpdateServiceInput {
            metadata: None,
            service_id: service.id.clone(),
            multi_person: Some(ServiceMultiPersonOptions::Group(
                max_booking_spots + booking_spots_inc,
            )),
        };
        admin_client
            .service
            .update(input)
            .await
            .expect("To update service");

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
        let available_slot = bookingslots[0].slots[0].start;

        for _ in 0..booking_spots_inc - 1 {
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
            assert_eq!(
                booking_intend
                    .selected_hosts
                    .iter()
                    .map(|h| h.id.clone())
                    .collect::<Vec<_>>(),
                vec![host.id.clone()]
            );
            assert_eq!(booking_intend.create_event_for_hosts, false);
        }
        // And now the last spot
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
        assert_eq!(
            booking_intend
                .selected_hosts
                .iter()
                .map(|h| h.id.clone())
                .collect::<Vec<_>>(),
            vec![host.id.clone()]
        );
        assert_eq!(booking_intend.create_event_for_hosts, true);
    }
}

#[actix_web::main]
#[test]
async fn test_group_team_scheduling_decrease_max_count() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let test_set = vec![(5, 2), (1, 1), (1, 0), (10, 4)];
    for (max_booking_spots, booking_spots_dec) in test_set {
        let input = CreateServiceInput {
            metadata: None,
            multi_person: Some(ServiceMultiPersonOptions::Group(max_booking_spots)),
        };
        let service = admin_client
            .service
            .create(input)
            .await
            .expect("To create service")
            .service;

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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
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

        let available_slot = bookingslots[0].slots[0].start;

        for _ in 0..max_booking_spots - 1 {
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
            assert_eq!(
                booking_intend
                    .selected_hosts
                    .iter()
                    .map(|h| h.id.clone())
                    .collect::<Vec<_>>(),
                vec![host.id.clone()]
            );
            assert_eq!(booking_intend.create_event_for_hosts, false);
        }
        // Last spot
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
        assert_eq!(
            booking_intend
                .selected_hosts
                .iter()
                .map(|h| h.id.clone())
                .collect::<Vec<_>>(),
            vec![host.id.clone()]
        );
        assert_eq!(booking_intend.create_event_for_hosts, true);
        let service_event = CreateEventInput {
            busy: Some(true),
            calendar_id: busy_calendar.id.clone(),
            duration,
            metadata: None,
            recurrence: None,
            reminder: None,
            service_id: Some(service.id.clone()),
            start_ts: available_slot,
        };
        let service_event = admin_client
            .event
            .create(host.id.clone(), service_event)
            .await
            .expect("To create service event")
            .event;

        // Now there are no more spots available on that timestamp
        // Lets decrease max count and check that it still is not possible to change
        let input = UpdateServiceInput {
            metadata: None,
            service_id: service.id.clone(),
            multi_person: Some(ServiceMultiPersonOptions::Group(
                max_booking_spots - booking_spots_dec,
            )),
        };
        admin_client
            .service
            .update(input)
            .await
            .expect("To update service");

        // The current service event should still be there
        let input = GetEventInput {
            event_id: service_event.id.clone(),
        };
        assert!(admin_client.event.get(input).await.is_ok());
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
            .expect_err("Booking should be fullbooked");
    }
}
