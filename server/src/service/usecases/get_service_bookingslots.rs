use crate::{api::Context, calendar::{domain::date, usecases::get_user_freebusy::GetUserFreeBusyUseCase}, event::domain::booking_slots::{BookingQueryError, BookingSlotsOptions, BookingSlotsQuery, ServiceBookingSlot, ServiceBookingSlotDTO, get_service_bookingslots, validate_bookingslots_query, validate_slots_interval}, shared::auth::ensure_nettu_acct_header};
use crate::{
    event::domain::booking_slots::UserFreeEvents,
    shared::usecase::{perform, Usecase},
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PathParams {
    service_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    iana_tz: Option<String>,
    duration: i64,
    interval: i64,
    date: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIRes {
    booking_slots: Vec<ServiceBookingSlotDTO>,
}

pub async fn get_service_bookingslots_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let _account = match ensure_nettu_acct_header(&http_req) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let usecase = GetServiceBookingSlotsUseCase {
        service_id: path_params.service_id.clone(),
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
        interval: query_params.interval,
    };

    let res = perform(usecase, &ctx).await;

    match res {
        Ok(r) => {
            let res = APIRes {
                booking_slots: r
                    .booking_slots
                    .iter()
                    .map(|slot| ServiceBookingSlotDTO::new(slot))
                    .collect(),
            };
            HttpResponse::Ok().json(res)
        }
        Err(e) => match e {
            UseCaseErrors::InvalidDateError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UseCaseErrors::InvalidTimezoneError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            UseCaseErrors::InvalidIntervalError => {
                HttpResponse::UnprocessableEntity().body(
                    "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds."
                )
            }
            UseCaseErrors::ServiceNotFoundError => HttpResponse::NotFound().finish(),
        },
    }
}

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
impl Usecase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        if !validate_slots_interval(self.interval) {
            return Err(UseCaseErrors::InvalidIntervalError);
        }

        let query = BookingSlotsQuery {
            date: self.date.clone(),
            iana_tz: self.iana_tz.clone(),
            interval: self.interval.clone(),
            duration: self.duration.clone(),
        };
        let booking_timespan = match validate_bookingslots_query(&query) {
            Ok(t) => t,
            Err(e) => match e {
                BookingQueryError::InvalidIntervalError => return Err(UseCaseErrors::InvalidIntervalError),
                BookingQueryError::InvalidDateError(d) => return Err(UseCaseErrors::InvalidDateError(d)),
                BookingQueryError::InvalidTimezoneError(d) => return Err(UseCaseErrors::InvalidTimezoneError(d)),
            }
        };

        let service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(s) => s,
            None => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        let mut users_freebusy: Vec<UserFreeEvents> = Vec::with_capacity(service.users.len());

        for user in &service.users {
            let usecase = GetUserFreeBusyUseCase {
                calendar_ids: Some(user.calendar_ids.clone()),
                end_ts: booking_timespan.end_ts,
                start_ts: booking_timespan.start_ts,
                user_id: user.user_id.clone(),
            };

            let free_events = perform(usecase, &ctx).await;

            match free_events {
                Ok(free_events) => {
                    users_freebusy.push(UserFreeEvents {
                        free_events: free_events.free,
                        user_id: user.user_id.clone(),
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
