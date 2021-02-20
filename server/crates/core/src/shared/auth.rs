use serde::{Deserialize, Serialize};

use crate::shared::usecase::execute;
use crate::{
    context::Context,
    user::{domain::User, usecases::create_user::CreateUserUseCase},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: usize,      // Expiration time (as UTC timestamp)
    iat: usize,      // Issued at (as UTC timestamp)
    user_id: String, // Subject (whom token refers to)
    scheduler_policy: Option<Policy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    allow: Option<Vec<Permission>>,
    reject: Option<Vec<Permission>>,
}

impl Policy {
    pub fn authorize(&self, permissions: &Vec<Permission>) -> bool {
        if permissions.is_empty() {
            return true;
        }

        if let Some(rejected) = &self.reject {
            for permission in permissions {
                if *permission == Permission::All {
                    return false;
                }
                if rejected.contains(permission) {
                    return false;
                }
            }
        }

        if let Some(allowed) = &self.allow {
            // First loop to check if All exists
            if allowed.contains(&Permission::All) {
                return true;
            }

            for permission in permissions {
                if !allowed.contains(permission) {
                    return false;
                }
            }
        }

        false
    }

    pub fn empty() -> Self {
        Self {
            allow: None,
            reject: None,
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    #[serde(rename = "*")]
    All,
    CreateCalendar,
    DeleteCalendar,
    UpdateCalendar,
    CreateCalendarEvent,
    DeleteCalendarEvent,
    UpdateCalendarEvent,
    CreateSchedule,
    UpdateSchedule,
    DeleteSchedule,
}
