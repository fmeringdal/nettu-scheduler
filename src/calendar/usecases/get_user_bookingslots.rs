use super::get_user_freebusy::{GetUserFreeBusyReq, GetUserFreeBusyUseCase};
use crate::{
    event::domain::booking_slots::{get_booking_slots, BookingSlot, BookingSlotsOptions},
    shared::usecase::UseCase,
};
use async_trait::async_trait;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct GetUserBookingSlotsReq {
    pub user_id: String,
    pub calendar_ids: Option<Vec<String>>,
    pub date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
}

pub struct GetUserBookingSlotsUseCase {
    pub get_user_freebusy_usecase: Arc<GetUserFreeBusyUseCase>,
}

#[derive(Serialize)]
pub struct GetUserBookingSlotsResponse {
    booking_slots: Vec<BookingSlot>,
}

#[async_trait(?Send)]
impl UseCase<GetUserBookingSlotsReq, Result<GetUserBookingSlotsResponse, GetUserBookingSlotsErrors>>
    for GetUserBookingSlotsUseCase
{
    async fn execute(
        &self,
        req: GetUserBookingSlotsReq,
    ) -> Result<GetUserBookingSlotsResponse, GetUserBookingSlotsErrors> {
        let tz: Result<Tz, _> = req.iana_tz.unwrap_or(String::from("UTC")).parse();
        if tz.is_err() {
            // handle this
        }
        let dates = req.date.split("-").collect::<Vec<_>>();
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

        println!("Date i got: {}", date);

        let start_of_day = date.and_hms(0, 0, 0);
        let end_of_day = (date + Duration::days(1)).and_hms(0, 0, 0);

        let free_events = self
            .get_user_freebusy_usecase
            .execute(GetUserFreeBusyReq {
                calendar_ids: req.calendar_ids,
                end_ts: end_of_day.timestamp_millis(),
                start_ts: start_of_day.timestamp_millis(),
                user_id: req.user_id,
            })
            .await;

        match free_events {
            Ok(free_events) => {
                println!("Free events???");
                println!("{:?}", free_events.free);
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
            Err(e) => panic!("djsaojdosa"), // ! fix this
        }
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
