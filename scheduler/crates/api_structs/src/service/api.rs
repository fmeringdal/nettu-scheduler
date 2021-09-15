use crate::dtos::{ServiceDTO, ServiceResourceDTO, ServiceWithUsersDTO};
use nettu_scheduler_domain::{
    BusyCalendar, Service, ServiceResource, ServiceWithUsers, TimePlan, Tz, ID,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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
        pub availability: Option<TimePlan>,
        #[serde(default)]
        pub buffer_after: Option<i64>,
        #[serde(default)]
        pub buffer_before: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResourceDTO;
}

pub mod add_busy_calendar {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    pub struct RequestBody {
        pub busy: BusyCalendar,
    }

    pub type APIResponse = String;
}

pub mod remove_busy_calendar {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    pub struct RequestBody {
        pub busy: BusyCalendar,
    }

    pub type APIResponse = String;
}

pub mod remove_service_event_intend {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    pub struct QueryParams {
        pub timestamp: i64,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct APIResponse {
        message: String,
    }

    impl Default for APIResponse {
        fn default() -> Self {
            Self {
                message: "Deleted Booking Intend".into(),
            }
        }
    }
}

pub mod create_service_event_intend {
    use super::*;
    use crate::dtos::UserDTO;
    use nettu_scheduler_domain::User;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub host_user_ids: Option<Vec<ID>>,
        pub timestamp: i64,
        pub duration: i64,
        pub interval: i64,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub selected_hosts: Vec<UserDTO>,
        pub create_event_for_hosts: bool,
    }

    impl APIResponse {
        pub fn new(selected_hosts: Vec<User>, create_event_for_hosts: bool) -> Self {
            Self {
                selected_hosts: selected_hosts.into_iter().map(UserDTO::new).collect(),
                create_event_for_hosts,
            }
        }
    }
}

pub mod create_service {
    use nettu_scheduler_domain::{Metadata, ServiceMultiPersonOptions};

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub metadata: Option<Metadata>,
        #[serde(default)]
        pub multi_person: Option<ServiceMultiPersonOptions>,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod update_service {
    use nettu_scheduler_domain::{Metadata, ServiceMultiPersonOptions};

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub metadata: Option<Metadata>,
        #[serde(default)]
        pub multi_person: Option<ServiceMultiPersonOptions>,
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
        pub timezone: Option<Tz>,
        pub duration: i64,
        pub interval: i64,
        pub start_date: String,
        pub end_date: String,
        #[serde(default)]
        pub host_user_ids: Option<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
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

    #[derive(Deserialize, Serialize, Debug)]
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

    #[derive(Deserialize, Serialize, Debug)]
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
        pub availability: Option<TimePlan>,
        pub buffer_after: Option<i64>,
        pub buffer_before: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResourceDTO;
}
