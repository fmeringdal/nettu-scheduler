pub struct ServiceBookingOption {
    duration: usize
}

pub struct ServiceResource {
    user_id: String,
}

pub struct Service {
    id: String,
    booking_options: Vec<ServiceBookingOption>,
    interval: usize,
    // allow_more_booking_requests_in_queue_than_resources
    // breaks / buffer
    // max_per_day
    users: Vec<ServiceResource>,
    // metadata ?
}