use crate::{
    shared::{
        entity::{Entity, ID},
        metadata::Metadata,
    },
    Meta,
};
use chrono_tz::{Tz, UTC};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub settings: CalendarSettings,
    pub metadata: Metadata,
}

impl Meta for Calendar {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone)]
pub struct CalendarSettings {
    pub week_start: isize,
    pub timezone: Tz,
}

impl CalendarSettings {
    pub fn set_week_start(&mut self, wkst: isize) -> bool {
        if wkst >= 0 && wkst <= 6 {
            self.week_start = wkst;
            true
        } else {
            false
        }
    }

    pub fn set_timezone(&mut self, timezone: &str) -> bool {
        match timezone.parse::<Tz>() {
            Ok(tzid) => {
                self.timezone = tzid;
                true
            }
            Err(_) => false,
        }
    }
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            week_start: 0,
            timezone: UTC,
        }
    }
}

impl Calendar {
    pub fn new(user_id: &ID, account_id: &ID) -> Self {
        Self {
            id: Default::default(),
            user_id: user_id.clone(),
            account_id: account_id.clone(),
            settings: Default::default(),
            metadata: Default::default(),
        }
    }
}

impl Entity for Calendar {
    fn id(&self) -> &ID {
        &self.id
    }
}
