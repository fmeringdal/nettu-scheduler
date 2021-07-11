use crate::scheduling::RoundRobinAlgorithm;
use crate::{
    shared::entity::{Entity, ID},
    Meta, Metadata,
};
use serde::{Deserialize, Serialize};

/// A type that describes a time plan and is either a `Calendar` or a `Schedule`
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "id")]
pub enum TimePlan {
    /// Calendar id
    Calendar(ID),
    /// Schedule id
    Schedule(ID),
    // No plan
    Empty,
}

/// A bookable `User` registered on a `Service`
#[derive(Clone, Debug, Serialize)]
pub struct ServiceResource {
    /// Id of the `User` registered on this `Service`
    pub user_id: ID,
    /// Id of the `Service` this user is reqistered on
    pub service_id: ID,
    /// Every available event in a `Calendar` or a `Schedule` in this field
    /// describes the time when this `ServiceResource` will be bookable.
    /// Note: If there are busy `CalendarEvent`s in the `Calendar` then the user
    /// will not be bookable during that time.
    pub availability: TimePlan,
    /// List of `Calendar` ids that should be subtracted from the availability
    /// time plan.
    pub busy: Vec<BusyCalendar>,
    /// This `ServiceResource` will not be bookable this amount of *minutes*
    /// after a meeting. A `CalendarEvent` will be interpreted as a meeting
    /// if the attribute `services` on the `CalendarEvent` includes this
    /// `Service` id.
    pub buffer_after: i64,
    /// This `ServiceResource` will not be bookable this amount of *minutes*
    /// before a meeting.
    pub buffer_before: i64,
    /// Minimum amount of time in minutes before this user could receive any
    /// booking requests. That means that if a bookingslots query is made at
    /// time T then this `ServiceResource` will not have any available
    /// bookingslots before at least T + `closest_booking_time`
    pub closest_booking_time: i64,
    /// Amount of time in minutes into the future after which the user can not receive any
    /// booking requests. This is useful to ensure that booking requests are not made multiple
    /// years into the future. That means that if a bookingslots query is made at
    /// time T then this `ServiceResource` will not have any available
    /// bookingslots after T + `furthest_booking_time`
    pub furthest_booking_time: Option<i64>,
}

impl ServiceResource {
    pub fn new(
        user_id: ID,
        service_id: ID,
        availability: TimePlan,
        busy: Vec<BusyCalendar>,
    ) -> Self {
        Self {
            service_id,
            user_id,
            availability,
            busy,
            buffer_after: 0,
            buffer_before: 0,
            closest_booking_time: 0,
            furthest_booking_time: None,
        }
    }

    pub fn set_availability(&mut self, availability: TimePlan) {
        self.availability = availability;
    }

    pub fn set_busy(&mut self, busy: Vec<BusyCalendar>) {
        self.busy = busy;
    }

    fn valid_buffer(&self, buffer: i64) -> bool {
        let min_buffer = 0;
        let max_buffer = 60 * 12; // 12 Hours
        if buffer < min_buffer || buffer > max_buffer {
            return false;
        }
        true
    }

    pub fn set_buffer_after(&mut self, buffer: i64) -> bool {
        if self.valid_buffer(buffer) {
            self.buffer_after = buffer;
            return true;
        }
        false
    }

    pub fn set_buffer_before(&mut self, buffer: i64) -> bool {
        if self.valid_buffer(buffer) {
            self.buffer_before = buffer;
            return true;
        }
        false
    }

    pub fn get_schedule_id(&self) -> Option<ID> {
        match &self.availability {
            TimePlan::Schedule(id) => Some(id.clone()),
            _ => None,
        }
    }

    pub fn contains_calendar(&self, calendar_id: &str) -> bool {
        match &self.availability {
            TimePlan::Calendar(id) if id.to_string() == calendar_id => {
                return true;
            }
            _ => (),
        }

        for busy in &self.busy {
            match busy {
                BusyCalendar::Nettu(id) if id.to_string() == calendar_id => return true,
                BusyCalendar::Google(id) if id == calendar_id => return true,
                _ => (),
            }
        }

        false
    }

    pub fn remove_calendar(&mut self, calendar_id: &str) {
        match &self.availability {
            TimePlan::Calendar(id) if id.to_string() == calendar_id => {
                self.availability = TimePlan::Empty;
            }
            _ => (),
        }

        self.busy.retain(|busy_cal| match busy_cal {
            BusyCalendar::Nettu(cal_id) => cal_id.to_string() != calendar_id,
            BusyCalendar::Google(cal_id) => cal_id != calendar_id,
            BusyCalendar::Outlook(cal_id) => cal_id != calendar_id,
        });
    }

    pub fn contains_schedule(&self, schedule_id: &ID) -> bool {
        matches!(&self.availability, TimePlan::Schedule(id) if id == schedule_id)
    }

    pub fn remove_schedule(&mut self, schedule_id: &ID) {
        match &self.availability {
            TimePlan::Schedule(id) if id == schedule_id => {
                self.availability = TimePlan::Empty;
            }
            _ => (),
        }
    }
}

impl Entity<String> for ServiceResource {
    fn id(&self) -> String {
        format!("{}#{}", self.service_id, self.user_id)
    }
}

#[derive(Clone, Debug)]
pub struct Service {
    pub id: ID,
    pub account_id: ID,
    // interval: usize,
    pub multi_person: ServiceMultiPersonOptions,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data", rename_all = "camelCase")]
pub enum ServiceMultiPersonOptions {
    RoundRobinAlgorithm(RoundRobinAlgorithm),
    Collective,
    Group(usize),
}

impl Default for ServiceMultiPersonOptions {
    fn default() -> Self {
        Self::RoundRobinAlgorithm(RoundRobinAlgorithm::default())
    }
}

impl Entity<ID> for Service {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

impl Meta<ID> for Service {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

impl Service {
    pub fn new(account_id: ID) -> Self {
        Self {
            id: Default::default(),
            account_id,
            multi_person: Default::default(),
            metadata: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct ServiceWithUsers {
    pub id: ID,
    pub account_id: ID,
    pub users: Vec<ServiceResource>,
    pub multi_person: ServiceMultiPersonOptions,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "provider", content = "id")]
pub enum BusyCalendar {
    Google(String),
    Outlook(String),
    Nettu(ID),
}

/// Relevant for group services
#[derive(Debug)]
pub struct ServiceReservation {
    pub id: ID,
    pub timestamp: i64,
    pub service_id: ID,
}
