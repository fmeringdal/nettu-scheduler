use chrono::{DateTime, Utc};

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    // https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html
    // 2001-07-08
    dt.format("%F").to_string()
}
