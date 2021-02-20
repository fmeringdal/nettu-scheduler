use crate::{
    calendar::usecases::get_user_freebusy::GetUserFreeBusyUseCase,
    context::Context,
    event::domain::booking_slots::{
        get_service_bookingslots, validate_bookingslots_query, validate_slots_interval,
        BookingQueryError, BookingSlotsOptions, BookingSlotsQuery, ServiceBookingSlot,
    },
};
use crate::{
    event::domain::booking_slots::UserFreeEvents,
    shared::usecase::{execute, UseCase},
};

use futures::future::join_all;
use serde::{Deserialize, Serialize};

struct GetServiceBookingSlotsUseCase {
    pub service_id: String,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
}

struct UseCaseRes {
    booking_slots: Vec<ServiceBookingSlot>,
}

#[derive(Debug)]
enum UseCaseErrors {
    ServiceNotFoundError,
    InvalidIntervalError,
    InvalidDateError(String),
    InvalidTimezoneError(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        if !validate_slots_interval(self.interval) {
            return Err(UseCaseErrors::InvalidIntervalError);
        }

        let query = BookingSlotsQuery {
            date: self.date.clone(),
            iana_tz: self.iana_tz.clone(),
            interval: self.interval,
            duration: self.duration,
        };
        let booking_timespan = match validate_bookingslots_query(&query) {
            Ok(t) => t,
            Err(e) => match e {
                BookingQueryError::InvalidIntervalError => {
                    return Err(UseCaseErrors::InvalidIntervalError)
                }
                BookingQueryError::InvalidDateError(d) => {
                    return Err(UseCaseErrors::InvalidDateError(d))
                }
                BookingQueryError::InvalidTimezoneError(d) => {
                    return Err(UseCaseErrors::InvalidTimezoneError(d))
                }
            },
        };

        let service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(s) => s,
            None => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        let mut users_freebusy: Vec<UserFreeEvents> = Vec::with_capacity(service.users.len());

        let mut usecase_futures = vec![];

        for user in &service.users {
            let usecase = GetUserFreeBusyUseCase {
                calendar_ids: Some(user.calendar_ids.clone()),
                schedule_ids: Some(user.schedule_ids.clone()),
                end_ts: booking_timespan.end_ts,
                start_ts: booking_timespan.start_ts,
                user_id: user.user_id.clone(),
            };

            usecase_futures.push(execute(usecase, &ctx));
        }

        let users_free_events = join_all(usecase_futures).await;
        for user_free_events in users_free_events {
            match user_free_events {
                Ok(free_events) => {
                    users_freebusy.push(UserFreeEvents {
                        free_events: free_events.free,
                        user_id: free_events.user_id,
                    });
                }
                Err(e) => {
                    println!("Error getting user freebusy: {:?}", e);
                }
            }
        }

        let booking_slots = get_service_bookingslots(
            users_freebusy,
            &BookingSlotsOptions {
                interval: self.interval,
                duration: self.duration,
                end_ts: booking_timespan.end_ts,
                start_ts: booking_timespan.start_ts,
            },
        );

        Ok(UseCaseRes { booking_slots })
    }
}

#[cfg(test)]
mod test {
    use chrono::prelude::*;
    use chrono::Utc;

    use crate::{
        calendar::domain::Calendar,
        event::domain::event::{CalendarEvent, RRuleOptions},
        service::domain::{Service, ServiceResource},
    };

    use super::*;

    struct TestContext {
        ctx: context::Context,
        service: Service,
    }

    async fn setup() -> TestContext {
        let ctx = Context::create_inmemory();

        let service = Service::new("123".into());
        ctx.repos.service_repo.insert(&service).await.unwrap();

        TestContext { ctx, service }
    }

    async fn setup_service_users(ctx: &Context, service: &mut Service) {
        let mut resource1 = ServiceResource {
            calendar_ids: vec![],
            schedule_ids: vec![],
            id: "1".into(),
            user_id: "1".into(),
        };
        let mut resource2 = ServiceResource {
            calendar_ids: vec![],
            schedule_ids: vec![],
            id: "2".into(),
            user_id: "2".into(),
        };

        let calendar_user_1 = Calendar::new(&resource1.user_id);
        resource1.calendar_ids.push(calendar_user_1.id.clone());
        let calendar_user_2 = Calendar::new(&resource2.user_id);
        resource2.calendar_ids.push(calendar_user_2.id.clone());

        ctx.repos
            .calendar_repo
            .insert(&calendar_user_1)
            .await
            .unwrap();
        ctx.repos
            .calendar_repo
            .insert(&calendar_user_2)
            .await
            .unwrap();

        let availibility_event1 = CalendarEvent {
            busy: false,
            calendar_id: calendar_user_1.id,
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: vec![],
            id: "1".into(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            account_id: "1".into(),
            user_id: resource1.user_id.to_owned(),
            reminder: None,
        };
        let availibility_event2 = CalendarEvent {
            busy: false,
            calendar_id: calendar_user_2.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: vec![],
            id: "2".into(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            account_id: "1".into(),
            user_id: resource2.user_id.to_owned(),
            reminder: None,
        };
        let mut availibility_event3 = CalendarEvent {
            busy: false,
            calendar_id: calendar_user_2.id,
            duration: 1000 * 60 * 105,
            end_ts: 0,
            exdates: vec![],
            id: "3".into(),
            recurrence: None,
            start_ts: 1000 * 60 * 60 * 4,
            user_id: resource1.user_id.to_owned(),
            account_id: "1".into(),
            reminder: None,
        };
        let recurrence = RRuleOptions {
            ..Default::default()
        };
        availibility_event3.set_recurrence(recurrence, &calendar_user_2.settings, true);

        ctx.repos
            .event_repo
            .insert(&availibility_event1)
            .await
            .unwrap();
        ctx.repos
            .event_repo
            .insert(&availibility_event2)
            .await
            .unwrap();
        ctx.repos
            .event_repo
            .insert(&availibility_event3)
            .await
            .unwrap();

        service.add_user(resource1);
        service.add_user(resource2);
        ctx.repos.service_repo.save(&service).await.unwrap();
    }

    #[actix_rt::main]
    #[test]
    async fn get_service_bookingslots() {
        let TestContext { ctx, service } = setup().await;

        let mut usecase = GetServiceBookingSlotsUseCase {
            date: "2010-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        assert!(res.unwrap().booking_slots.is_empty());
    }

    #[actix_rt::main]
    #[test]
    async fn get_bookingslots_with_multiple_users_in_service() {
        let TestContext { ctx, mut service } = setup().await;
        setup_service_users(&ctx, &mut service).await;

        let mut usecase = GetServiceBookingSlotsUseCase {
            date: "2010-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id.clone(),
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.len(), 4);
        for i in 0..4 {
            assert_eq!(booking_slots[i].duration, usecase.duration);
            assert_eq!(booking_slots[i].user_ids, vec!["2"]);
            assert_eq!(
                booking_slots[i].start,
                Utc.ymd(2010, 1, 1)
                    .and_hms(4, 15 * i as u32, 0)
                    .timestamp_millis()
            );
        }

        let mut usecase = GetServiceBookingSlotsUseCase {
            date: "1970-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.len(), 5);
        assert_eq!(booking_slots[0].user_ids, vec!["1", "2"]);
        for i in 0..5 {
            assert_eq!(booking_slots[i].duration, usecase.duration);
            if i > 0 {
                assert_eq!(booking_slots[i].user_ids, vec!["2"]);
                assert_eq!(
                    booking_slots[i].start,
                    Utc.ymd(1970, 1, 1)
                        .and_hms(4, 15 * (i - 1) as u32, 0)
                        .timestamp_millis()
                );
            }
        }
    }
}
