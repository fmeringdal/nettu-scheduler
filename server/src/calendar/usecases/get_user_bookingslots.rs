use super::get_user_freebusy::GetUserFreeBusyUseCase;
use crate::{
    api::Context,
    event::domain::booking_slots::{
        get_booking_slots, validate_bookingslots_query, BookingQueryError, BookingSlot,
        BookingSlotsOptions, BookingSlotsQuery,
    },
    shared::{
        auth::ensure_nettu_acct_header,
        usecase::{execute, Usecase},
    },
    user::domain::User,
};
use actix_web::{web, HttpRequest, HttpResponse};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    external_user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserBookingQuery {
    iana_tz: Option<String>,
    duration: i64,
    interval: i64,
    date: String,
    calendar_ids: Option<String>,
}

pub async fn get_user_bookingslots_controller(
    http_req: HttpRequest,
    query_params: web::Query<UserBookingQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let account = match ensure_nettu_acct_header(&http_req) {
        Ok(a) => a,
        Err(e) => return e,
    };
    let calendar_ids = match &query_params.calendar_ids {
        Some(calendar_ids) => Some(calendar_ids.split(',').map(String::from).collect()),
        None => None,
    };

    let _user_id = User::create_id(&account, &params.external_user_id);

    let usecase = GetUserBookingSlotsUsecase {
        user_id: User::create_id(&account, &params.external_user_id),
        calendar_ids,
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
        interval: query_params.interval,
    };

    let res = execute(usecase, &ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
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
            UseCaseErrors::UserFreebusyError => HttpResponse::InternalServerError().finish(),
        },
    }
}

pub struct GetUserBookingSlotsUsecase {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseResponse {
    booking_slots: Vec<BookingSlot>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    InvalidDateError(String),
    InvalidTimezoneError(String),
    InvalidIntervalError,
    UserFreebusyError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for GetUserBookingSlotsUsecase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
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

        let freebusy_usecase = GetUserFreeBusyUseCase {
            calendar_ids: self.calendar_ids.clone(),
            end_ts: booking_timespan.end_ts,
            start_ts: booking_timespan.start_ts,
            user_id: self.user_id.clone(),
        };
        let free_events = execute(freebusy_usecase, ctx).await;

        match free_events {
            Ok(free_events) => {
                let booking_slots = get_booking_slots(
                    &free_events.free,
                    &BookingSlotsOptions {
                        interval: self.interval,
                        duration: self.duration,
                        end_ts: booking_timespan.end_ts,
                        start_ts: booking_timespan.start_ts,
                    },
                );

                Ok(UseCaseResponse { booking_slots })
            }
            Err(_e) => Err(UseCaseErrors::UserFreebusyError),
        }
    }
}
