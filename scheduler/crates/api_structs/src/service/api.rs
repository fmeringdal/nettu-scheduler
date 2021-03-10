use nettu_scheduler_domain::{Service, TimePlan, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::ServiceDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResponse {
    pub service: ServiceDTO,
}

impl ServiceResponse {
    pub fn new(service: Service) -> Self {
        Self {
            service: ServiceDTO::new(service),
        }
    }
}

pub mod add_user_to_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub user_id: ID,
        pub availibility: Option<TimePlan>,
        pub busy: Option<Vec<ID>>,
        pub buffer: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod create_service {
    use nettu_scheduler_domain::Metadata;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod update_service {
    use nettu_scheduler_domain::Metadata;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub metadata: Option<Metadata>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod get_service_bookingslots {
    use super::*;
    use nettu_scheduler_domain::booking_slots::ServiceBookingSlot;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub iana_tz: Option<String>,
        pub duration: i64,
        pub interval: i64,
        pub date: String,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ServiceBookingSlotDTO {
        pub start: i64,
        pub duration: i64,
        pub user_ids: Vec<ID>,
    }

    impl ServiceBookingSlotDTO {
        pub fn new(slot: ServiceBookingSlot) -> Self {
            Self {
                duration: slot.duration,
                start: slot.start,
                user_ids: slot.user_ids,
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub booking_slots: Vec<ServiceBookingSlotDTO>,
    }

    impl APIResponse {
        pub fn new(booking_slots: Vec<ServiceBookingSlot>) -> Self {
            Self {
                booking_slots: booking_slots
                    .into_iter()
                    .map(|slot| ServiceBookingSlotDTO::new(slot))
                    .collect(),
            }
        }
    }
}

pub mod get_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod get_services_by_meta {
    use super::*;

    #[derive(Deserialize)]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Serialize)]
    pub struct APIResponse {
        pub services: Vec<ServiceDTO>,
    }

    impl APIResponse {
        pub fn new(services: Vec<Service>) -> Self {
            Self {
                services: services.into_iter().map(|s| ServiceDTO::new(s)).collect(),
            }
        }
    }
}

pub mod delete_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod remove_user_from_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod update_service_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub availibility: Option<TimePlan>,
        pub busy: Option<Vec<ID>>,
        pub buffer: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResponse;
}
