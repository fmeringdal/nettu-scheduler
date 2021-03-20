use crate::error::NettuError;
use crate::shared::usecase::{execute, UseCase};
use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use nettu_scheduler_api_structs::get_service_bookingslots::*;
use nettu_scheduler_domain::{
    booking_slots::{
        get_service_bookingslots, validate_bookingslots_query, validate_slots_interval,
        BookingQueryError, BookingSlotsOptions, BookingSlotsQuery, ServiceBookingSlot,
        UserFreeEvents,
    },
    get_free_busy, Calendar, CompatibleInstances, EventInstance, ServiceResource, TimePlan,
    TimeSpan, ID,
};
use nettu_scheduler_infra::NettuContext;
use tracing::warn;

pub async fn get_service_bookingslots_controller(
    _http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let usecase = GetServiceBookingSlotsUseCase {
        service_id: path_params.service_id.clone(),
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
        interval: query_params.interval,
    };

    execute(usecase, &ctx).await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.booking_slots))
        })
        .map_err(|e| match e {
            UseCaseErrors::InvalidDate(msg) => {
                NettuError::BadClientData(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UseCaseErrors::InvalidTimezone(msg) => {
                NettuError::BadClientData(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            UseCaseErrors::InvalidInterval => {
                NettuError::BadClientData(
                    "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds.".into()
                )
            }
            UseCaseErrors::InvalidTimespan => {
                NettuError::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
            UseCaseErrors::ServiceNotFound => NettuError::NotFound(format!("Service with id: {}, was not found.", path_params.service_id)),
        })
}

#[derive(Debug)]
struct GetServiceBookingSlotsUseCase {
    pub service_id: ID,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Debug)]
struct UseCaseRes {
    booking_slots: Vec<ServiceBookingSlot>,
}

#[derive(Debug)]
enum UseCaseErrors {
    ServiceNotFound,
    InvalidInterval,
    InvalidTimespan,
    InvalidDate(String),
    InvalidTimezone(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "GetServiceBookingSlots";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        if !validate_slots_interval(self.interval) {
            return Err(UseCaseErrors::InvalidInterval);
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
                BookingQueryError::InvalidInterval => return Err(UseCaseErrors::InvalidInterval),
                BookingQueryError::InvalidDate(d) => return Err(UseCaseErrors::InvalidDate(d)),
                BookingQueryError::InvalidTimezone(d) => {
                    return Err(UseCaseErrors::InvalidTimezone(d))
                }
            },
        };

        let service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(s) => s,
            None => return Err(UseCaseErrors::ServiceNotFound),
        };

        let mut usecase_futures: Vec<_> = Vec::with_capacity(service.users.len());

        let timespan = TimeSpan::new(booking_timespan.start_ts, booking_timespan.end_ts);
        if timespan.greater_than(ctx.config.booking_slots_query_duration_limit) {
            return Err(UseCaseErrors::InvalidTimespan);
        }

        for user in &service.users {
            let timespan = timespan.clone();
            usecase_futures.push(self.get_bookable_times(user, timespan, ctx));
        }

        let users_free_events = join_all(usecase_futures).await;

        let booking_slots = get_service_bookingslots(
            users_free_events,
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

impl GetServiceBookingSlotsUseCase {
    async fn get_user_availibility(
        &self,
        user: &ServiceResource,
        user_calendars: &[Calendar],
        timespan: &TimeSpan,
        ctx: &NettuContext,
    ) -> CompatibleInstances {
        let empty = CompatibleInstances::new(vec![]);
        match &user.availibility {
            TimePlan::Calendar(id) => {
                let calendar = match user_calendars.iter().find(|cal| cal.id == *id) {
                    Some(cal) => cal,
                    None => {
                        return empty;
                    }
                };
                let all_calendar_events = ctx
                    .repos
                    .event_repo
                    .find_by_calendar(&id, Some(&timespan))
                    .await
                    .unwrap_or_default();

                let all_event_instances = all_calendar_events
                    .iter()
                    .map(|e| e.expand(Some(&timespan), &calendar.settings))
                    .flatten()
                    .collect::<Vec<_>>();

                get_free_busy(all_event_instances).free
            }
            TimePlan::Schedule(id) => match ctx.repos.schedule_repo.find(&id).await {
                Some(schedule) if schedule.user_id == user.user_id => schedule.freebusy(&timespan),
                _ => empty,
            },
            TimePlan::Empty => empty,
        }
    }

    async fn get_user_busy(
        &self,
        user: &ServiceResource,
        busy_calendars: &[&Calendar],
        timespan: &TimeSpan,
        ctx: &NettuContext,
    ) -> CompatibleInstances {
        let mut busy_events: Vec<EventInstance> = vec![];

        for cal in busy_calendars {
            match ctx
                .repos
                .event_repo
                .find_by_calendar(&cal.id, Some(&timespan))
                .await
            {
                Ok(calendar_events) => {
                    let mut calendar_busy_events = calendar_events
                        .into_iter()
                        .filter(|e| e.busy)
                        .map(|e| {
                            let mut instances = e.expand(Some(&timespan), &cal.settings);

                            // Add buffer to instances if event is a service event
                            if user.buffer > 0 && e.is_service {
                                let buffer_in_millis = user.buffer * 60 * 1000;
                                for instance in instances.iter_mut() {
                                    instance.end_ts += buffer_in_millis;
                                }
                            }

                            instances
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    busy_events.append(&mut calendar_busy_events);
                }
                Err(e) => {
                    warn!("Unable to fetch user calendars: {}", e);
                }
            }
        }

        // This should be optimized later
        CompatibleInstances::new(busy_events)
    }

    /// Ensure that calendar timespan fits within user settings for when
    /// it should be bookable
    fn parse_calendar_timespan(
        user: &ServiceResource,
        mut timespan: TimeSpan,
        ctx: &NettuContext,
    ) -> Result<TimeSpan, ()> {
        let first_available =
            ctx.sys.get_timestamp_millis() + user.closest_booking_time * 60 * 1000;
        if timespan.start() < first_available {
            timespan = TimeSpan::new(first_available, timespan.end());
        }
        if let Some(furthest_booking_time) = user.furthest_booking_time {
            let last_available = furthest_booking_time * 60 * 1000 + ctx.sys.get_timestamp_millis();
            if last_available < timespan.end() {
                if last_available <= timespan.start() {
                    return Err(());
                }
                timespan = TimeSpan::new(timespan.start(), last_available);
            }
        }

        if timespan.greater_than(ctx.config.booking_slots_query_duration_limit) {
            Err(())
        } else {
            Ok(timespan)
        }
    }

    /// Finds the bookable times for a `User`.
    async fn get_bookable_times(
        &self,
        user: &ServiceResource,
        mut timespan: TimeSpan,
        ctx: &NettuContext,
    ) -> UserFreeEvents {
        let empty = UserFreeEvents {
            free_events: CompatibleInstances::new(vec![]),
            user_id: user.id.clone(),
        };

        match Self::parse_calendar_timespan(user, timespan, ctx) {
            Ok(parsed_timespan) => timespan = parsed_timespan,
            Err(_) => return empty,
        }

        let user_calendars = ctx.repos.calendar_repo.find_by_user(&user.user_id).await;
        let busy_calendars = user_calendars
            .iter()
            .filter(|cal| user.busy.contains(&cal.id))
            .collect::<Vec<_>>();

        let mut free_events = self
            .get_user_availibility(user, &user_calendars, &timespan, ctx)
            .await;

        let busy_events = self
            .get_user_busy(user, &busy_calendars, &timespan, ctx)
            .await;

        free_events.remove_intances(&busy_events, 0);

        UserFreeEvents {
            free_events,
            user_id: user.id.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use chrono::prelude::*;
    use chrono::Utc;
    use nettu_scheduler_domain::{Calendar, CalendarEvent, RRuleOptions, Service, ServiceResource};
    use nettu_scheduler_infra::{setup_context, ISys};

    struct TestContext {
        ctx: NettuContext,
        service: Service,
    }

    struct DummySys {}

    impl ISys for DummySys {
        fn get_timestamp_millis(&self) -> i64 {
            0
        }
    }

    async fn setup() -> TestContext {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(DummySys {});

        let service = Service::new(Default::default());
        ctx.repos.service_repo.insert(&service).await.unwrap();

        TestContext { ctx, service }
    }

    async fn setup_service_users(ctx: &NettuContext, service: &mut Service) {
        let mut resource1 = ServiceResource {
            id: Default::default(),
            user_id: Default::default(),
            buffer: 0,
            availibility: TimePlan::Empty,
            busy: vec![],
            closest_booking_time: 0,
            furthest_booking_time: None,
        };
        let mut resource2 = ServiceResource {
            id: Default::default(),
            user_id: Default::default(),
            buffer: 0,
            availibility: TimePlan::Empty,
            busy: vec![],
            closest_booking_time: 0,
            furthest_booking_time: None,
        };

        let account_id = ID::default();

        let calendar_user_1 = Calendar::new(&resource1.user_id, &account_id);
        resource1.availibility = TimePlan::Calendar(calendar_user_1.id.clone());
        let calendar_user_2 = Calendar::new(&resource2.user_id, &account_id);
        resource2.availibility = TimePlan::Calendar(calendar_user_2.id.clone());

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

        let account_id = ID::default();

        let availibility_event1 = CalendarEvent {
            id: Default::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_1.id,
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: vec![],
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: resource1.user_id.to_owned(),
            reminder: None,
            is_service: false,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        let availibility_event2 = CalendarEvent {
            id: ID::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_2.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: vec![],
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: resource2.user_id.to_owned(),
            reminder: None,
            is_service: false,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
        };
        let mut availibility_event3 = CalendarEvent {
            id: ID::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_2.id,
            duration: 1000 * 60 * 105,
            end_ts: 0,
            exdates: vec![],
            recurrence: None,
            start_ts: 1000 * 60 * 60 * 4,
            user_id: resource1.user_id.to_owned(),
            reminder: None,
            is_service: false,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
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

    #[actix_web::main]
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

    #[actix_web::main]
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
            assert_eq!(booking_slots[i].user_ids.len(), 1);
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
        assert_eq!(booking_slots[0].user_ids.len(), 2);
        for i in 0..5 {
            assert_eq!(booking_slots[i].duration, usecase.duration);
            if i > 0 {
                assert_eq!(booking_slots[i].user_ids.len(), 1);
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
