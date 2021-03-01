use nettu_scheduler_domain::{Service, ServiceResource, TimePlan, User};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub id: String,
    pub user_id: String,
    pub availibility: TimePlan,
    pub busy: Vec<String>,
    pub buffer: i64,
}

impl ServiceResourceDTO {
    pub fn new(resource: &ServiceResource) -> Self {
        Self {
            id: resource.id.clone(),
            user_id: User::create_external_id(&resource.user_id),
            availibility: resource.availibility.clone(),
            busy: resource.busy.clone(),
            buffer: resource.buffer,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDTO {
    pub id: String,
    pub account_id: String,
    pub users: Vec<ServiceResourceDTO>,
}

impl ServiceDTO {
    pub fn new(service: &Service) -> Self {
        Self {
            id: service.id.clone(),
            account_id: service.account_id.clone(),
            users: service
                .users
                .iter()
                .map(|u| ServiceResourceDTO::new(u))
                .collect(),
        }
    }
}
