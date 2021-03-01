use chrono::Utc;

// Mocking out time so that it
// is possible to run tests depending
// that check the current timestamp
pub trait ISys: Send + Sync {
    /// The current timestamp in millis
    fn get_timestamp_millis(&self) -> i64;
}

pub struct RealSys {}
impl ISys for RealSys {
    fn get_timestamp_millis(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}
