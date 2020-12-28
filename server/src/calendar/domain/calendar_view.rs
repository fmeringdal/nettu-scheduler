use chrono::prelude::*;
use chrono::DateTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarView {
    start_ts: i64,
    end_ts: i64,
}

#[derive(Debug)]
pub struct InvalidTimespanError(i64, i64);

impl Error for InvalidTimespanError {}

impl std::fmt::Display for InvalidTimespanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Provided timespan start_ts: {} and end_ts: {} is invalid. It should be between 1 hour and 40 days.", self.0, self.1)
    }
}

pub struct CalendarViewDateTime {
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}

impl CalendarView {
    pub fn create(start_ts: i64, end_ts: i64) -> Result<Self, InvalidTimespanError> {
        let max_timespan = 1000 * 60 * 60 * 24 * 40;
        let min_timespan = 1000 * 60 * 60;
        let delta = end_ts - start_ts;

        if delta > max_timespan || delta < min_timespan {
            println!("st: {}, end: {}", start_ts, end_ts);
            println!("Delta got: {}", delta);
            return Err(InvalidTimespanError(start_ts, end_ts));
        } else {
            return Ok(Self { start_ts, end_ts });
        }
    }

    pub fn as_datetime(&self, tz: &Tz) -> CalendarViewDateTime {
        CalendarViewDateTime {
            start: tz.timestamp(self.start_ts as i64 / 1000 + 1, 0),
            end: tz.timestamp(self.end_ts as i64 / 1000, 0),
        }
    }

    pub fn get_start(&self) -> i64 {
        self.start_ts
    }

    pub fn get_end(&self) -> i64 {
        self.end_ts
    }
}
