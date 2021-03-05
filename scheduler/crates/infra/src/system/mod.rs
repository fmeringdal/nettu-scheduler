use chrono::Utc;

// Mocking out time so that it is possible to run tests that depend on time.
pub trait ISys: Send + Sync {
    /// The current timestamp in millis
    fn get_timestamp_millis(&self) -> i64;
}

/// System that gets the real time and is used when not testing
pub struct RealSys {}
impl ISys for RealSys {
    fn get_timestamp_millis(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
}
