use super::event_instance::EventInstance;
use chrono::prelude::*;
use chrono_tz::{Tz, UTC};
use rrule::{Frequenzy, ParsedOptions, RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Calendar {
    pub id: String,
    pub user_id: String,
}
