use crate::shared::entity::Entity;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// A type that describes a time plan and is either a `Calendar` og a `Schedule`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "variant", content = "id")]
pub enum Plan {
    /// Calendar id
    Calendar(String),
    /// Schedule id
    Schedule(String),
    // No plan
    Empty,
}

/// A bookable `User` registered on a `Service`
#[derive(Clone, Debug, Serialize)]
pub struct ServiceResource {
    pub id: String,
    /// Id of the `User` registered on this `Service`
    pub user_id: String,
    /// Every available event in a `Calendar` or
    /// any `Shedule` in this field describes the time when this user
    /// will be bookable.
    /// Note: if there are busy `CalendarEvent`s in the `Calendar` then the user
    /// will not be bookable during that time.
    pub availibility: Plan,
    /// Any `Calendar` that should be subtracted from
    /// the availibility field.
    pub busy: Vec<String>,
    /// The user will not be bookable
    /// this amount of *minutes* after a meeting.
    /// A `CalendarEvent` will be interpreted as a meeting
    /// if the attribute `metadata.service_resources` includes
    /// this `ServiceResource` id.
    pub buffer: usize,
    // max_per_day
}

impl ServiceResource {
    pub fn new(user_id: &str, availibility: Plan, busy: Vec<String>) -> Self {
        Self {
            id: ObjectId::new().to_string(),
            user_id: String::from(user_id),
            availibility,
            busy,
            buffer: 0,
        }
    }

    pub fn set_availibility(&mut self, availibility: Plan) {
        self.availibility = availibility;
    }

    pub fn set_busy(&mut self, busy: Vec<String>) {
        self.busy = busy;
    }

    pub fn set_buffer(&mut self, buffer: usize) -> bool {
        let min_buffer = 0;
        let max_buffer = 60 * 12; // 12 Hours
        if buffer < min_buffer || buffer > max_buffer {
            return false;
        }
        self.buffer = buffer;
        true
    }

    pub fn get_calendar_ids(&self) -> Vec<String> {
        let mut calendar_ids = self.busy.clone();

        match &self.availibility {
            Plan::Calendar(id) => {
                calendar_ids.push(id.clone());
            }
            _ => (),
        };

        calendar_ids
    }

    pub fn get_schedule_id(&self) -> Option<String> {
        match &self.availibility {
            Plan::Schedule(id) => Some(id.clone()),
            _ => None,
        }
    }

    pub fn contains_calendar(&self, calendar_id: &str) -> bool {
        match &self.availibility {
            Plan::Calendar(id) if id == calendar_id => {
                return true;
            }
            _ => (),
        }

        self.busy.contains(&String::from(calendar_id))
    }

    pub fn remove_calendar(&mut self, calendar_id: &str) {
        match &self.availibility {
            Plan::Calendar(id) if id == calendar_id => {
                self.availibility = Plan::Empty;
            }
            _ => (),
        }

        self.busy.retain(|cal_id| cal_id != calendar_id);
    }

    pub fn contains_schedule(&self, schedule_id: &str) -> bool {
        match &self.availibility {
            Plan::Schedule(id) if id == schedule_id => true,
            _ => false,
        }
    }

    pub fn remove_schedule(&mut self, schedule_id: &str) {
        match &self.availibility {
            Plan::Schedule(id) if id == schedule_id => {
                self.availibility = Plan::Empty;
            }
            _ => (),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Service {
    pub id: String,
    pub account_id: String,
    // interval: usize,
    // allow_more_booking_requests_in_queue_than_resources
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
