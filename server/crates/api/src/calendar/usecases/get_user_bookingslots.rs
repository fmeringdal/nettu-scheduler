use super::get_user_freebusy::GetUserFreeBusyUseCase;
use crate::{
    error::NettuError,
    shared::{
        auth::ensure_nettu_acct_header,
        usecase::{execute, UseCase},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{
    booking_slots::{
        get_booking_slots, validate_bookingslots_query, BookingQueryError, BookingSlot,
        BookingSlotsOptions, BookingSlotsQuery,
    },
    User,
};

use nettu_scheduler_infra::NettuContext;
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
    schedule_ids: Option<String>,
}

pub async fn get_user_bookingslots_controller(
    http_req: HttpRequest,
    query_params: web::Query<UserBookingQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = ensure_nettu_acct_header(&http_req)?;
    let calendar_ids = match &query_params.calendar_ids {
        Some(ids) => Some(ids.split(',').map(String::from).collect()),
        None => None,
    };
    let schedule_ids = match &query_params.schedule_ids {
        Some(ids) => Some(ids.split(',').map(String::from).collect()),
        None => None,
    };

    let _user_id = User::create_id(&account, &params.external_user_id);

    let usecase = GetUserBookingSlotsUseCase {
        user_id: User::create_id(&account, &params.external_user_id),
        calendar_ids,
        schedule_ids,
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
        interval: query_params.interval,
    };

    execute(usecase, &ctx).await
        .map(|usecase_res| HttpResponse::Ok().json(usecase_res))
        .map_err(|e| match e {
            UseCaseErrors::InvalidDateError(msg) => {
                NettuError::BadClientData(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UseCaseErrors::InvalidTimezoneError(msg) => {
                NettuError::BadClientData(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            UseCaseErrors::InvalidIntervalError => {
                NettuError::BadClientData(
                    "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds.".into()
                )
            }
            UseCaseErrors::UserFreebusyError => NettuError::InternalError,
        })
}

pub struct GetUserBookingSlotsUseCase {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub schedule_ids: Option<Vec<String>>,
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
impl UseCase for GetUserBookingSlotsUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

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
            schedule_ids: self.schedule_ids.clone(),
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
