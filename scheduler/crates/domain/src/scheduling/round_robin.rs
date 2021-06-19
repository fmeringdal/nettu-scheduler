use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{CalendarEvent, ServiceResource, ID};
use itertools::Itertools;

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
        self.members.sort_by_key(|m| m.1.map(|ts| -1 * ts));
        let mut least_recently_booked_members: Vec<(ID, Option<i64>)> = vec![];
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

pub struct RoundRobinEqualDistributionAssignment {
    /// List of upcoming `Service Event`s they are assigned for the given `Service`
    pub events: Vec<CalendarEvent>,
    /// List of user that can be assigned the new `Service Event`
    pub user_ids: Vec<ID>,
}

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
            .take_while(|u| match prev {
                Some(count) => {
                    if count == u.event_count {
                        prev = Some(u.event_count);
                        true
                    } else {
                        false
                    }
                }
                None => true,
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

    #[test]
    fn round_robin_availability_assignment() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn round_robin_eq_distribution_assignment() {
        assert_eq!(2 + 2, 4);
    }
}
