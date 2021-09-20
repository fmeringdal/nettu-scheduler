use chrono::{DateTime, Utc};
use nettu_scheduler_sdk::User;

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    // https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html
    // 2001-07-08
    dt.format("%F").to_string()
}

pub fn assert_equal_user_lists(users1: &[User], users2: &[User]) {
    assert_eq!(users1.len(), users2.len());
    let mut users1 = users1.to_owned();
    users1.sort_by_key(|u| u.id.to_string());
    let mut users2 = users2.to_owned();
    users2.sort_by_key(|u| u.id.to_string());
    for (user1, user2) in users1.iter().zip(users2) {
        assert_eq!(user1.id, user2.id);
    }
}
