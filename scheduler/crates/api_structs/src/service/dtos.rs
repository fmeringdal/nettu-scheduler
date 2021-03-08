use nettu_scheduler_domain::{Service, ServiceResource, TimePlan, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub id: ID,
    pub user_id: ID,
    pub availibility: TimePlan,
    pub busy: Vec<ID>,
    pub buffer: i64,
    pub closest_booking_time: i64,
    pub furthest_booking_time: Option<i64>,
}

impl ServiceResourceDTO {
    pub fn new(resource: ServiceResource) -> Self {
        Self {
            id: resource.id.clone(),
            user_id: resource.user_id.clone(),
            availibility: resource.availibility,
            busy: resource.busy.clone(),
            buffer: resource.buffer,
            closest_booking_time: resource.closest_booking_time,
            furthest_booking_time: resource.furthest_booking_time,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDTO {
    pub id: ID,
    pub account_id: ID,
    pub users: Vec<ServiceResourceDTO>,
}

impl ServiceDTO {
    pub fn new(service: Service) -> Self {
        Self {
            id: service.id.clone(),
            account_id: service.account_id.clone(),
            users: service
                .users
                .into_iter()
                .map(|u| ServiceResourceDTO::new(u))
                .collect(),
        }
    }
}
