use crate::{
    api::Context,
    calendar::{domain::date, usecases::get_user_freebusy::GetUserFreeBusyUseCase},
    event::domain::booking_slots::{
        get_service_bookingslots, validate_slots_interval, BookingSlotsOptions, ServiceBookingSlot,
        ServiceBookingSlotDTO,
    },
    shared::auth::ensure_nettu_acct_header,
};
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
            UsecaseErrors::InvalidDateError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UsecaseErrors::InvalidTimezoneError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            UsecaseErrors::InvalidIntervalError => {
                HttpResponse::UnprocessableEntity().body(
                    "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds."
                )
            }
            UsecaseErrors::ServiceNotFoundError => HttpResponse::NotFound().finish(),
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
enum UsecaseErrors {
    ServiceNotFoundError,
    InvalidIntervalError,
    InvalidDateError(String),
    InvalidTimezoneError(String),
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Errors = UsecaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        if !validate_slots_interval(self.interval) {
            return Err(UsecaseErrors::InvalidIntervalError);
        }

        let iana_tz = self.iana_tz.clone().unwrap_or(String::from("UTC"));
        let tz: Tz = match iana_tz.parse() {
            Ok(tz) => tz,
            Err(_) => return Err(UsecaseErrors::InvalidTimezoneError(iana_tz)),
        };

        let parsed_date = match date::is_valid_date(&self.date) {
            Ok(val) => val,
            Err(_) => return Err(UsecaseErrors::InvalidDateError(self.date.clone())),
        };

        let date = tz.ymd(parsed_date.0, parsed_date.1, parsed_date.2);

        let start_of_day = date.and_hms(0, 0, 0);
        let end_of_day = (date + Duration::days(1)).and_hms(0, 0, 0);

        let service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(s) => s,
            None => return Err(UsecaseErrors::ServiceNotFoundError),
        };

        let mut users_freebusy: Vec<UserFreeEvents> = Vec::with_capacity(service.users.len());

        for user in &service.users {
            let usecase = GetUserFreeBusyUseCase {
                calendar_ids: Some(user.calendar_ids.clone()),
                end_ts: end_of_day.timestamp_millis(),
                start_ts: start_of_day.timestamp_millis(),
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
                end_ts: end_of_day.timestamp_millis(),
                start_ts: start_of_day.timestamp_millis(),
            },
        );

        Ok(UseCaseRes { booking_slots })
    }
}
