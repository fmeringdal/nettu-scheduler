use nettu_scheduler_core::{Service, ServiceResource, User};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
    pub schedule_ids: Vec<String>,
}

impl ServiceResourceDTO {
    pub fn new(resource: &ServiceResource) -> Self {
        Self {
            id: resource.id.clone(),
            calendar_ids: resource.calendar_ids.clone(),
            schedule_ids: resource.schedule_ids.clone(),
            user_id: User::create_external_id(&resource.user_id),
        }
    }
}

#[derive(Serialize)]
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
