use crate::{CalendarEvent, ID};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

/// Round robin algorithm to decide which member should be assigned a
/// `Service Event` when there are multiple members of a `Service`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RoundRobinAlgorithm {
    /// Optimizes for availability
    ///
    /// This assigns the `Service Event` to the member which was
    /// least recently assigned a `Service Event` for the given
    /// `Service`.
    Availability,
    /// Optimizes for equal distribution
    ///
    /// This assigns the `Service Event` to the member which was
    /// least number of assigned `Service Event`s for the next
    /// time period. Time period in this context is hard coded to be
    /// two weeks.
    EqualDistribution,
}

impl Default for RoundRobinAlgorithm {
    fn default() -> Self {
        Self::Availability
    }
}

#[derive(Debug, Clone)]
pub struct RoundRobinAvailabilityAssignment {
    /// List of members with a corresponding timestamp stating
    /// when the they were assigned a `Service Event` last time, if they have
    /// been assigned
    pub members: Vec<(ID, Option<i64>)>,
}

impl RoundRobinAvailabilityAssignment {
    pub fn assign(mut self) -> Option<ID> {
        if self.members.is_empty() {
            return None;
        }
        self.members.sort_by_key(|m| m.1);
        let mut least_recently_booked_members: Vec<(ID, Option<i64>)> = Vec::new();
        for member in self.members {
            if least_recently_booked_members.is_empty()
                || member.1 == least_recently_booked_members[0].1
            {
                least_recently_booked_members.push(member);
            } else {
                break;
            }
        }

        if least_recently_booked_members.len() == 1 {
            Some(least_recently_booked_members[0].0.clone())
        } else {
            // Just pick random
            let mut rng = thread_rng();
            let rand_user_index = rng.gen_range(0..least_recently_booked_members.len());
            Some(least_recently_booked_members[rand_user_index].0.clone())
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoundRobinEqualDistributionAssignment {
    /// List of upcoming `Service Event`s they are assigned for the given `Service`
    pub events: Vec<CalendarEvent>,
    /// List of user that can be assigned the new `Service Event`
    pub user_ids: Vec<ID>,
}

#[derive(Debug)]
struct UserWithEvents {
    pub user_id: ID,
    pub event_count: usize,
}

impl RoundRobinEqualDistributionAssignment {
    pub fn assign(self) -> Option<ID> {
        let mut prev: Option<usize> = None;
        let users_with_least_upcoming_bookings = self
            .user_ids
            .iter()
            .map(|user_id| UserWithEvents {
                event_count: self.events.iter().filter(|e| &e.user_id == user_id).count(),
                user_id: user_id.clone(),
            })
            .sorted_by_key(|u| u.event_count)
            .take_while(|u| {
                let take = match prev {
                    Some(count) => count == u.event_count,
                    None => true,
                };
                prev = Some(u.event_count);
                take
            })
            .collect::<Vec<_>>();

        if self.user_ids.is_empty() {
            None
        } else if users_with_least_upcoming_bookings.len() == 1 {
            Some(users_with_least_upcoming_bookings[0].user_id.clone())
        } else {
            // Just pick random
            let mut rng = thread_rng();
            let rand_user_index = rng.gen_range(0..users_with_least_upcoming_bookings.len());
            Some(
                users_with_least_upcoming_bookings[rand_user_index]
                    .user_id
                    .clone(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::SliceRandom;

    use super::*;

    #[test]
    fn round_robin_availability_assignment_without_members() {
        let query = RoundRobinAvailabilityAssignment {
            members: Vec::new(),
        };
        assert!(query.assign().is_none());
    }

    #[test]
    fn round_robin_availability_assignment() {
        let none_user_1 = ID::default();
        let none_user_2 = ID::default();
        let none_user_3 = ID::default();
        let none_user_4 = ID::default();
        let members = vec![
            (none_user_4.clone(), None),
            (ID::default(), Some(10)),
            (none_user_1.clone(), None),
            (ID::default(), Some(6)),
            (ID::default(), Some(12)),
            (ID::default(), Some(20)),
            (ID::default(), Some(0)),
            (ID::default(), Some(-28)),
            (none_user_2.clone(), None),
            (none_user_3.clone(), None),
        ];
        let none_user_ids = vec![none_user_1, none_user_2, none_user_3, none_user_4];
        let query = RoundRobinAvailabilityAssignment { members };
        assert!(query.clone().assign().is_some());
        let selected_member = query.clone().assign().unwrap();
        assert!(none_user_ids.contains(&selected_member));

        // Check that random member is selected when there are multiple that are possible to select
        let prev = selected_member;
        let mut found_other = false;
        for _ in 0..100 {
            let selected_member = query.clone().assign().unwrap();
            if selected_member != prev {
                found_other = true;
                break;
            }
        }
        assert!(found_other)
    }

    #[test]
    fn round_robin_availability_assignment_2() {
        let user_1 = ID::default();
        let members = vec![
            (ID::default(), Some(10)),
            (user_1.clone(), Some(4)),
            (ID::default(), Some(6)),
            (ID::default(), Some(12)),
            (ID::default(), Some(20)),
            (ID::default(), Some(100)),
            (ID::default(), Some(28)),
        ];
        let query = RoundRobinAvailabilityAssignment { members };
        assert!(query.clone().assign().is_some());
        let selected_member = query.clone().assign().unwrap();
        assert_eq!(selected_member, user_1);
    }

    #[test]
    fn round_robin_eq_distribution_assignment_without_members() {
        let query = RoundRobinEqualDistributionAssignment {
            events: Vec::new(),
            user_ids: Vec::new(),
        };
        assert!(query.assign().is_none());
    }

    fn generate_default_event(user_id: &ID) -> CalendarEvent {
        CalendarEvent {
            id: Default::default(),
            start_ts: Default::default(),
            duration: Default::default(),
            busy: Default::default(),
            end_ts: Default::default(),
            created: Default::default(),
            updated: Default::default(),
            recurrence: Default::default(),
            exdates: Default::default(),
            calendar_id: Default::default(),
            user_id: user_id.clone(),
            account_id: Default::default(),
            reminders: Default::default(),
            service_id: Default::default(),
            metadata: Default::default(),
        }
    }

    struct UserWithEventsCount {
        pub user_id: ID,
        pub count: usize,
    }

    impl UserWithEventsCount {
        pub fn new(count: usize) -> Self {
            Self {
                user_id: Default::default(),
                count,
            }
        }
    }

    #[test]
    fn round_robin_eq_distribution_assignment() {
        let least_bookings = 1;
        let user_with_events_count = vec![
            UserWithEventsCount::new(least_bookings),
            UserWithEventsCount::new(least_bookings + 100),
            UserWithEventsCount::new(least_bookings + 1),
            UserWithEventsCount::new(least_bookings),
            UserWithEventsCount::new(least_bookings),
            UserWithEventsCount::new(least_bookings + 12),
            UserWithEventsCount::new(least_bookings),
        ];
        let users_with_least_upcoming_bookings = user_with_events_count
            .iter()
            .filter(|u| u.count == least_bookings)
            .map(|u| u.user_id.clone())
            .collect::<Vec<_>>();

        let user_ids = user_with_events_count
            .iter()
            .map(|u| u.user_id.clone())
            .collect::<Vec<_>>();
        let mut events = user_with_events_count
            .iter()
            .map(|u| {
                let mut user_events = Vec::with_capacity(u.count);
                for _ in 0..u.count {
                    user_events.push(generate_default_event(&u.user_id));
                }
                user_events
            })
            .flatten()
            .collect::<Vec<_>>();
        events.shuffle(&mut thread_rng());

        let query = RoundRobinEqualDistributionAssignment { events, user_ids };
        assert!(query.clone().assign().is_some());
        let selected_member = query.clone().assign().unwrap();
        assert!(users_with_least_upcoming_bookings.contains(&selected_member));

        // Check that random member is selected when there are multiple that are possible to select
        let prev = selected_member;
        let mut found_other = false;
        for _ in 0..100 {
            let selected_member = query.clone().assign().unwrap();
            if selected_member != prev {
                found_other = true;
                break;
            }
        }
        assert!(found_other)
    }
}
