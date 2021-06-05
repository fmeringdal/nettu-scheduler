use nettu_scheduler_domain::{Service, ServiceResource, ServiceWithUsers, TimePlan, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::{ServiceDTO, ServiceResourceDTO, ServiceWithUsersDTO};

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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWithUsersResponse {
    pub service: ServiceWithUsersDTO,
}

impl ServiceWithUsersResponse {
    pub fn new(service: ServiceWithUsers) -> Self {
        Self {
            service: ServiceWithUsersDTO::new(service),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceResponse {
    pub user: ServiceResourceDTO,
}

impl ServiceResourceResponse {
    pub fn new(user: ServiceResource) -> Self {
        Self {
            user: ServiceResourceDTO::new(user),
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
        #[serde(default)]
        pub buffer: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResourceDTO;
}

pub mod create_service {
    use nettu_scheduler_domain::Metadata;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
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
        #[serde(default)]
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
    use nettu_scheduler_domain::booking_slots::{
        ServiceBookingSlot, ServiceBookingSlots, ServiceBookingSlotsDate,
    };

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
        pub start_date: String,
        pub end_date: String,
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
    pub struct ServiceBookingSlotsDateDTO {
        pub date: String,
        pub slots: Vec<ServiceBookingSlotDTO>,
    }

    impl ServiceBookingSlotsDateDTO {
        pub fn new(date_slots: ServiceBookingSlotsDate) -> Self {
            Self {
                date: date_slots.date,
                slots: date_slots
                    .slots
                    .into_iter()
                    .map(ServiceBookingSlotDTO::new)
                    .collect(),
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub dates: Vec<ServiceBookingSlotsDateDTO>,
    }

    impl APIResponse {
        pub fn new(booking_slots: ServiceBookingSlots) -> Self {
            Self {
                dates: booking_slots
                    .dates
                    .into_iter()
                    .map(ServiceBookingSlotsDateDTO::new)
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

    pub type APIResponse = ServiceWithUsersDTO;
}

pub mod get_services_by_meta {
    use crate::dtos::ServiceDTO;

    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResponse {
        pub services: Vec<ServiceDTO>,
    }

    impl APIResponse {
        pub fn new(services: Vec<Service>) -> Self {
            Self {
                services: services.into_iter().map(ServiceDTO::new).collect(),
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

    pub type APIResponse = String;
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

    pub type APIResponse = ServiceResourceDTO;
}
