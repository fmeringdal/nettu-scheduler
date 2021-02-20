use super::get_user_freebusy::GetUserFreeBusyUseCase;
use crate::{
    context::Context,
    event::domain::booking_slots::{
        get_booking_slots, validate_bookingslots_query, BookingQueryError, BookingSlot,
        BookingSlotsOptions, BookingSlotsQuery,
    },
    shared::usecase::{execute, UseCase},
};

use serde::Serialize;

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
