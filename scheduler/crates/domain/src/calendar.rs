use crate::{
    shared::{
        entity::{Entity, ID},
        metadata::Metadata,
    },
    IntegrationProvider, Meta, Weekday,
};
use chrono_tz::{Tz, UTC};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub settings: CalendarSettings,
    pub metadata: Metadata,
}

impl Meta<ID> for Calendar {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncedCalendar {
    pub provider: IntegrationProvider,
    pub calendar_id: ID,
    pub user_id: ID,
    pub ext_calendar_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSettings {
    pub week_start: Weekday,
    pub timezone: Tz,
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            week_start: Weekday::Mon,
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

impl Entity<ID> for Calendar {
    fn id(&self) -> ID {
        self.id.clone()
    }
}
