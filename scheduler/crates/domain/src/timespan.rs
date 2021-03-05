use chrono::prelude::*;
use chrono::DateTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// A `TimeSpan` type represents a time interval (duration of time)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSpan {
    start_ts: i64,
    end_ts: i64,
    duration: i64,
}

impl TimeSpan {
    pub fn new(start_ts: i64, end_ts: i64) -> Self {
        Self {
            start_ts,
            end_ts,
            duration: end_ts - start_ts,
        }
    }

    /// Duration of this `TimeSpan` is greater than a given duration
    pub fn greater_than(&self, duration: i64) -> bool {
        self.duration > duration
    }

    fn create_datetime_from_millis(timestamp_millis: i64, tz: &Tz) -> DateTime<Tz> {
        tz.timestamp_millis(timestamp_millis)
    }

    pub fn as_datetime(&self, tz: &Tz) -> TimeSpanDateTime {
        TimeSpanDateTime {
            start: TimeSpan::create_datetime_from_millis(self.start_ts, tz),
            end: TimeSpan::create_datetime_from_millis(self.end_ts, tz),
        }
    }

    pub fn start(&self) -> i64 {
        self.start_ts
    }

    pub fn end(&self) -> i64 {
        self.end_ts
    }
}

#[derive(Debug)]
pub struct InvalidTimeSpanError(i64, i64);

impl Error for InvalidTimeSpanError {}

impl std::fmt::Display for InvalidTimeSpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Provided timespan start_ts: {} and end_ts: {} is invalid. It should be between 1 hour and 40 days.", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct TimeSpanDateTime {
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}
