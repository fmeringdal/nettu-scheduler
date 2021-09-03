mod helpers;

use chrono::{Duration, Utc, Weekday};
use helpers::setup::spawn_app;
use helpers::utils::{assert_equal_user_lists, format_datetime};
use nettu_scheduler_domain::{BusyCalendar, ServiceMultiPersonOptions, TimePlan, ID};
use nettu_scheduler_sdk::{
    AddBusyCalendar, AddServiceUserInput, Calendar, CreateBookingIntendInput, CreateCalendarInput,
    CreateEventInput, CreateScheduleInput, CreateServiceInput, CreateUserInput, GetEventInput,
    GetServiceBookingSlotsInput, NettuSDK, UpdateServiceInput, User,
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
                assert_equal_user_lists(&booking_intend.selected_hosts, &hosts);
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
            assert_equal_user_lists(&booking_intend.selected_hosts, &hosts);
            assert_eq!(booking_intend.create_event_for_hosts, true);
            for (host, calendar) in hosts_with_calendar {
                let service_event = CreateEventInput {
                    busy: Some(true),
                    calendar_id: calendar.id.clone(),
                    duration,
                    metadata: None,
                    recurrence: None,
                    reminders: Vec::new(),
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
                timezone: Some(chrono_tz::UTC),
                end_date: format_datetime(&next_week),
                start_date: format_datetime(&tomorrow),
                host_user_ids: None,
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
        timezone: chrono_tz::UTC,
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
        timezone: Some(chrono_tz::UTC),
        end_date: format_datetime(&next_week),
        start_date: format_datetime(&tomorrow),
        host_user_ids: None,
    };
    let bookingslots = admin_client
        .service
        .bookingslots(input)
        .await
        .expect("To get bookingslots")
        .dates;

    // Host 1 is available but host 2 is never available
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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
        };
        admin_client
            .service
            .add_user(input)
            .await
            .expect("To add host to service");
        let input = AddBusyCalendar {
            user_id: host.id.clone(),
            service_id: service.id.clone(),
            calendar: BusyCalendar::Nettu(busy_calendar.id.clone()),
        };
        admin_client
            .service
            .add_busy_calendar(input)
            .await
            .expect("To add busy calendar to service user");

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
            reminders: Vec::new(),
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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
        };
        admin_client
            .service
            .add_user(input)
            .await
            .expect("To add host to service");
        let input = AddBusyCalendar {
            user_id: host.id.clone(),
            service_id: service.id.clone(),
            calendar: BusyCalendar::Nettu(busy_calendar.id.clone()),
        };
        admin_client
            .service
            .add_busy_calendar(input)
            .await
            .expect("To add busy calendar to service user");

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
            timezone: Some(chrono_tz::UTC),
            end_date: format_datetime(&next_week),
            start_date: format_datetime(&tomorrow),
            host_user_ids: None,
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
            service_id: service.id.clone(),
            user_id: host.id.clone(),
        };
        admin_client
            .service
            .add_user(input)
            .await
            .expect("To add host to service");
        let input = AddBusyCalendar {
            user_id: host.id.clone(),
            service_id: service.id.clone(),
            calendar: BusyCalendar::Nettu(busy_calendar.id.clone()),
        };
        admin_client
            .service
            .add_busy_calendar(input)
            .await
            .expect("To add busy calendar to service user");

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
            reminders: Vec::new(),
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

// When a reservation is made in a group service it should still allow more bookings
// to that service (if max invitees is more than one). But other services should then
// not allow bookings at that time
#[actix_web::main]
#[test]
async fn test_combination_of_services() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NettuSDK::new(address, res.secret_api_key);

    let input = CreateServiceInput {
        metadata: None,
        multi_person: Some(ServiceMultiPersonOptions::Group(10)),
    };
    let group_service = admin_client
        .service
        .create(input)
        .await
        .expect("To create service")
        .service;
    let input = CreateServiceInput {
        metadata: None,
        multi_person: Some(ServiceMultiPersonOptions::RoundRobinAlgorithm(
            Default::default(),
        )),
    };
    let round_robin_service = admin_client
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

    // Add user to both services
    for service_id in vec![round_robin_service.id.clone(), group_service.id.clone()] {
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
            service_id,
            calendar: BusyCalendar::Nettu(busy_calendar.id.clone()),
        };
        admin_client
            .service
            .add_busy_calendar(input)
            .await
            .expect("To add busy calendar to service user");
    }

    let tomorrow = Utc::now() + Duration::days(1);
    let next_week = tomorrow + Duration::days(7);
    let duration = 1000 * 60 * 30;
    let interval = 1000 * 60 * 30;
    let get_bookingslots_input = GetServiceBookingSlotsInput {
        duration,
        interval,
        service_id: group_service.id.clone(),
        timezone: Some(chrono_tz::UTC),
        end_date: format_datetime(&next_week),
        start_date: format_datetime(&tomorrow),
        host_user_ids: None,
    };
    let bookingslots = admin_client
        .service
        .bookingslots(get_bookingslots_input.clone())
        .await
        .expect("to get bookingslots")
        .dates;
    let mut get_round_robin_bookingslots_input = get_bookingslots_input.clone();
    get_round_robin_bookingslots_input.service_id = round_robin_service.id.clone();
    let bookingslots_round_robin = admin_client
        .service
        .bookingslots(get_round_robin_bookingslots_input.clone())
        .await
        .expect("to get bookingslots")
        .dates;

    let available_slot = bookingslots[0].slots[0].start;
    assert_eq!(available_slot, bookingslots_round_robin[0].slots[0].start);

    // Create booking intend for the group service
    let input = CreateBookingIntendInput {
        service_id: group_service.id.clone(),
        host_user_ids: None,
        timestamp: available_slot,
        duration,
        interval,
    };
    admin_client
        .service
        .create_booking_intend(input)
        .await
        .expect("To create booking intend");

    // And then create service event which is not busy
    let service_event = CreateEventInput {
        busy: Some(false),
        calendar_id: busy_calendar.id.clone(),
        duration,
        metadata: None,
        recurrence: None,
        reminders: Vec::new(),
        service_id: Some(group_service.id.clone()),
        start_ts: available_slot,
    };
    admin_client
        .event
        .create(host.id.clone(), service_event)
        .await
        .expect("To create service event");

    // Check that round robin no longer returns slots that is booked by the group event
    let bookingslots_round_robin = admin_client
        .service
        .bookingslots(get_round_robin_bookingslots_input.clone())
        .await
        .expect("to get bookingslots")
        .dates;
    assert!(available_slot < bookingslots_round_robin[0].slots[0].start);

    // But group service still returns the same available slot because there
    // are still more people that can book the group event
    let bookingslots = admin_client
        .service
        .bookingslots(get_bookingslots_input.clone())
        .await
        .expect("to get bookingslots")
        .dates;
    assert_eq!(available_slot, bookingslots[0].slots[0].start);
}
