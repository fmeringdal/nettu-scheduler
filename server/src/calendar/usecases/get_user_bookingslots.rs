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
    calendar_ids: Option<Vec<String>>,
}

pub async fn get_user_bookingslots_controller(
    query_params: web::Query<UserBookingQuery>,
    params: web::Path<UserPathParams>,
    ctx: web::Data<Context>,
) -> HttpResponse {
    let req = GetUserBookingSlotsReq {
        user_id: params.user_id.clone(),
        calendar_ids: query_params.calendar_ids.clone(),
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
            GetUserBookingSlotsErrors::InvalidTimespanError => {
                HttpResponse::UnprocessableEntity().finish()
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

async fn get_user_bookingslots_usecase(
    req: GetUserBookingSlotsReq,
    ctx: GetUserFreeBusyUseCaseCtx,
) -> Result<GetUserBookingSlotsResponse, GetUserBookingSlotsErrors> {
    let tz: Result<Tz, _> = req.iana_tz.unwrap_or(String::from("UTC")).parse();
    if tz.is_err() {
        // handle this
    }
    let dates = req.date.split('-').collect::<Vec<_>>();
    if dates.len() != 3 {
        // handle this
    }
    let year = dates[0].parse();
    let month = dates[1].parse();
    let day = dates[2].parse();

    if year.is_err() || month.is_err() || day.is_err() {
        // handle this
    }
    let year = year.unwrap();
    let month = month.unwrap();
    let day = day.unwrap();
    // handle invalid values for year month and day

    let date = tz.unwrap().ymd(year, month, day);

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
        Err(_e) => panic!("djsaojdosa"), // ! fix this
    }
}

#[derive(Debug)]
pub enum GetUserBookingSlotsErrors {
    InvalidTimespanError,
}

impl std::fmt::Display for GetUserBookingSlotsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GetUserBookingSlotsErrors::InvalidTimespanError => {
                write!(f, "The provided timesspan was invalid.")
            }
        }
    }
}
