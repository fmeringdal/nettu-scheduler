use super::get_user_freebusy::{
    get_user_freebusy_usecase, GetUserFreeBusyReq, GetUserFreeBusyUseCaseCtx,
};
use crate::{
    api::Context,
    event::domain::booking_slots::{get_booking_slots, BookingSlot, BookingSlotsOptions},
};
use actix_web::{web, HttpResponse};
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserPathParams {
    user_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserBookingQuery {
    iana_tz: Option<String>,
    duration: i64,
    date: String,
    calendar_ids: Option<String>,
}

pub async fn get_user_bookingslots_controller(
    query_params: web::Query<UserBookingQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let calendar_ids = match &query_params.calendar_ids {
        Some(calendar_ids) => Some(calendar_ids.split(",").map(|s| String::from(s)).collect()),
        None => None,
    };

    let req = GetUserBookingSlotsReq {
        user_id: params.user_id.clone(),
        calendar_ids,
        iana_tz: query_params.iana_tz.clone(),
        date: query_params.date.clone(),
        duration: query_params.duration,
    };
    let ctx = GetUserFreeBusyUseCaseCtx {
        event_repo: ctx.repos.event_repo.clone(),
        calendar_repo: ctx.repos.calendar_repo.clone(),
    };
    let res = get_user_bookingslots_usecase(req, ctx).await;

    match res {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            GetUserBookingSlotsErrors::InvalidDateError(msg) => HttpResponse::UnprocessableEntity()
                .body(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                )),
            GetUserBookingSlotsErrors::InvalidTimezoneError(msg) => {
                HttpResponse::UnprocessableEntity().body(format!(
                    "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                    msg
                ))
            }
            GetUserBookingSlotsErrors::UserFreebusyError => {
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetUserBookingSlotsReq {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserBookingSlotsResponse {
    booking_slots: Vec<BookingSlot>,
}

#[derive(Debug)]
pub enum GetUserBookingSlotsErrors {
    InvalidDateError(String),
    InvalidTimezoneError(String),
    UserFreebusyError,
}

async fn get_user_bookingslots_usecase(
    req: GetUserBookingSlotsReq,
    ctx: GetUserFreeBusyUseCaseCtx,
) -> Result<GetUserBookingSlotsResponse, GetUserBookingSlotsErrors> {
    let tz: Tz = match req.iana_tz.unwrap_or(String::from("UTC")).parse() {
        Ok(tz) => tz,
        Err(_) => return Err(GetUserBookingSlotsErrors::InvalidTimezoneError(req.date)),
    };

    let parsed_date = is_valid_date(&req.date)?;
    let date = tz.ymd(parsed_date.0, parsed_date.1, parsed_date.2);

    let start_of_day = date.and_hms(0, 0, 0);
    let end_of_day = (date + Duration::days(1)).and_hms(0, 0, 0);

    let free_events = get_user_freebusy_usecase(
        GetUserFreeBusyReq {
            calendar_ids: req.calendar_ids,
            end_ts: end_of_day.timestamp_millis(),
            start_ts: start_of_day.timestamp_millis(),
            user_id: req.user_id,
        },
        ctx,
    )
    .await;

    match free_events {
        Ok(free_events) => {
            println!(
                "Start: {}, end: {}",
                start_of_day.timestamp_millis(),
                end_of_day.timestamp_millis()
            );
            println!("Free events got: {:?}", free_events.free);
            let booking_slots = get_booking_slots(
                &free_events.free,
                BookingSlotsOptions {
                    interval: 1000 * 60 * 15, // 15 minutes
                    duration: req.duration,
                    end_ts: end_of_day.timestamp_millis(),
                    start_ts: start_of_day.timestamp_millis(),
                },
            );

            Ok(GetUserBookingSlotsResponse { booking_slots })
        }
        Err(_e) => Err(GetUserBookingSlotsErrors::UserFreebusyError),
    }
}

fn is_valid_date(datestr: &str) -> Result<(i32, u32, u32), GetUserBookingSlotsErrors> {
    let datestr = String::from(datestr);
    let dates = datestr.split('-').collect::<Vec<_>>();
    if dates.len() != 3 {
        return Err(GetUserBookingSlotsErrors::InvalidDateError(datestr));
    }
    let year = dates[0].parse();
    let month = dates[1].parse();
    let day = dates[2].parse();

    if year.is_err() || month.is_err() || day.is_err() {
        return Err(GetUserBookingSlotsErrors::InvalidDateError(datestr));
    }

    let year = year.unwrap();
    let month = month.unwrap();
    let day = day.unwrap();
    if year < 1970 || year > 2100 || month < 1 || month > 12 {
        return Err(GetUserBookingSlotsErrors::InvalidDateError(datestr));
    }

    let mut month_length = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if year % 400 == 0 || (year % 100 != 0 && year % 4 == 0) {
        month_length[1] = 29;
    }

    if day < 1 || day > month_length[month as usize] {
        return Err(GetUserBookingSlotsErrors::InvalidDateError(datestr));
    }

    Ok((year, month, day))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_accepts_valid_dates() {
        let valid_dates = vec!["2018-1-1"];

        for date in &valid_dates {
            assert!(is_valid_date(date).is_ok());
        }
    }

    #[test]
    fn it_rejects_invalid_dates() {
        let valid_dates = vec!["2018--1-1"];

        for date in &valid_dates {
            assert!(is_valid_date(date).is_err());
        }
    }
}
