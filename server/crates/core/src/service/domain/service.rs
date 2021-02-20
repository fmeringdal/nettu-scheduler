use mongodb::bson::oid::ObjectId;

use crate::{shared::entity::Entity, user::domain::User};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceResource {
    pub id: String,
    pub user_id: String,
    pub calendar_ids: Vec<String>,
    pub schedule_ids: Vec<String>,
}

impl ServiceResource {
    pub fn new(user_id: &str, calendar_ids: &[String], schedule_ids: &[String]) -> Self {
        Self {
            id: ObjectId::new().to_string(),
            user_id: String::from(user_id),
            calendar_ids: calendar_ids.to_owned(),
            schedule_ids: schedule_ids.to_owned(),
        }
    }

    pub fn set_calendar_ids(&mut self, calendar_ids: &[String]) {
        self.calendar_ids = calendar_ids.to_owned();
    }

    pub fn set_schedule_ids(&mut self, schedule_ids: &[String]) {
        self.schedule_ids = schedule_ids.to_owned();
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
