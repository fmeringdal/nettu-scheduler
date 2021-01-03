use mongodb::bson::oid::ObjectId;

use crate::{shared::entity::Entity, user::domain::User};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceResource {
    pub id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
}

impl ServiceResource {
    pub fn new(user_id: &str, calendar_ids: &Vec<String>) -> Self {
        Self {
            id: ObjectId::new().to_string(),
            user_id: String::from(user_id),
            calendar_ids: calendar_ids.clone(),
        }
    }

    pub fn set_calendar_ids(&mut self, calendar_ids: &Vec<String>) {
        self.calendar_ids = calendar_ids.to_owned();
    }
}

#[derive(Clone, Debug)]
pub struct Service {
    pub id: String,
    pub account_id: String,
    // interval: usize,
    // allow_more_booking_requests_in_queue_than_resources
    // breaks / buffer
    // max_per_day
    pub users: Vec<ServiceResource>,
    // metadata ?
}

impl Entity for Service {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Service {
    pub fn new(account_id: &str) -> Self {
        Self {
            id: ObjectId::new().to_string(),
            account_id: String::from(account_id),
            users: vec![],
        }
    }

    pub fn add_user(&mut self, user: ServiceResource) {
        self.users.push(user);
    }

    pub fn remove_user(&mut self, user_id: &str) -> Option<ServiceResource> {
        for (pos, user) in self.users.iter().enumerate() {
            if user.user_id == user_id {
                return Some(self.users.remove(pos));
            }
        }
        None
    }

    pub fn find_user(&self, user_id: &str) -> Option<&ServiceResource> {
        self.users.iter().find(|u| u.user_id == user_id)
    }

    pub fn find_user_mut(&mut self, user_id: &str) -> Option<&mut ServiceResource> {
        self.users.iter_mut().find(|u| u.user_id == user_id)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
}

impl ServiceResourceDTO {
    pub fn new(resource: &ServiceResource) -> Self {
        Self {
            id: resource.id.clone(),
            calendar_ids: resource.calendar_ids.clone(),
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
