use serde::{Deserialize, Serialize};

/// A Policy is set on a `User` and decides which actions it can and cannot take.
///
/// The `Policy` is created by the `Account` admin when creating the json web token
/// claims. Every `UseCase` contains a list of `Permission`s that is required
/// for a `User` to execute it, if the `User`s `Policy` is not authorized
/// some of these `Permission`s the request will be rejected.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Policy {
    /// `Permission`s allowed by the `Policy`
    allow: Option<Vec<Permission>>,
    /// `Permission`s rejected by the `Policy`
    reject: Option<Vec<Permission>>,
}

impl Policy {
    /// Checks if this `Policy` has the right to list of `Permission`s
    pub fn authorize(&self, permissions: &[Permission]) -> bool {
        if permissions.is_empty() {
            return true;
        }

        if let Some(rejected) = &self.reject {
            for rejected_permission in rejected {
                if *rejected_permission == Permission::All {
                    return false;
                }
                if permissions.contains(rejected_permission) {
                    return false;
                }
            }
        }

        if let Some(allowed) = &self.allow {
            // First loop to check if All exists
            if allowed.contains(&Permission::All) {
                return true;
            }

            // Check that all permissions are in allowed
            for permission in permissions {
                if !allowed.contains(permission) {
                    return false;
                }
            }

            return true;
        }

        false
    }
}

/// `Permission` are different kind of actions that can be performed.
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn permissions() {
        let policy = Policy::default();
        assert!(policy.authorize(&Vec::new()));
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: None,
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: Some(vec![Permission::CreateCalendar]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(Vec::new()),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(vec![Permission::CreateCalendar]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar]),
            reject: Some(vec![Permission::All]),
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar, Permission::UpdateCalendar]),
            reject: Some(vec![Permission::DeleteCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::UpdateCalendar
        ]));

        let policy = Policy {
            allow: Some(vec![Permission::UpdateCalendar]),
            reject: None,
        };
        assert!(!policy.authorize(&vec![Permission::CreateCalendar]));

        let policy = Policy {
            allow: Some(vec![Permission::CreateCalendar, Permission::UpdateCalendar]),
            reject: Some(vec![Permission::UpdateCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(!policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::UpdateCalendar
        ]));

        let policy = Policy {
            allow: Some(vec![Permission::All]),
            reject: Some(vec![Permission::UpdateCalendar]),
        };
        assert!(policy.authorize(&vec![Permission::CreateCalendar]));
        assert!(policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::DeleteCalendar,
        ]));
        assert!(!policy.authorize(&vec![
            Permission::CreateCalendar,
            Permission::DeleteCalendar,
            Permission::UpdateCalendar,
        ]));
    }
}
