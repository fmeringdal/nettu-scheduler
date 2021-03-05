use crate::shared::entity::{Entity, ID};
use chrono_tz::{Tz, UTC};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub id: ID,
    pub user_id: ID,
    pub settings: CalendarSettings,
}

#[derive(Debug, Clone)]
pub struct CalendarSettings {
    pub wkst: isize,
    pub timezone: Tz,
}

impl CalendarSettings {
    pub fn set_wkst(&mut self, wkst: isize) -> bool {
        if wkst >= 0 && wkst <= 6 {
            self.wkst = wkst;
            true
        } else {
            false
        }
    }

    pub fn set_timezone(&mut self, timezone: &String) -> bool {
        match timezone.parse::<Tz>() {
            Ok(tzid) => {
                self.timezone = tzid;
                true
            }
            Err(_) => false,
        }
    }
}

impl Calendar {
    pub fn new(user_id: &ID) -> Self {
        Self {
            id: Default::default(),
            user_id: user_id.clone(),
            settings: CalendarSettings {
                wkst: 0,
                timezone: UTC,
            },
        }
    }
}

impl Entity for Calendar {
    fn id(&self) -> &ID {
        &self.id
    }
}
