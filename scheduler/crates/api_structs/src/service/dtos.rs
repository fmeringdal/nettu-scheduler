use nettu_scheduler_domain::{BusyCalendar, Metadata, Service, ServiceResource, TimePlan, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub id: ID,
    pub user_id: ID,
    pub availibility: TimePlan,
    pub busy: Vec<BusyCalendar>,
    pub buffer: i64,
    pub closest_booking_time: i64,
    pub furthest_booking_time: Option<i64>,
}

impl ServiceResourceDTO {
    pub fn new(resource: ServiceResource) -> Self {
        Self {
            id: resource.id,
            user_id: resource.user_id,
            availibility: resource.availibility,
            busy: resource.busy,
            buffer: resource.buffer,
            closest_booking_time: resource.closest_booking_time,
            furthest_booking_time: resource.furthest_booking_time,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDTO {
    pub id: ID,
    pub users: Vec<ServiceResourceDTO>,
    pub metadata: Metadata,
}

impl ServiceDTO {
    pub fn new(service: Service) -> Self {
        Self {
            id: service.id.clone(),
            users: service
                .users
                .into_iter()
                .map(ServiceResourceDTO::new)
                .collect(),
            metadata: service.metadata,
        }
    }
}
