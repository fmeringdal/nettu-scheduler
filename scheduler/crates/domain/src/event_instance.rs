use crate::CalendarEvent;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Occurrence of a `CalendarEvent`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInstance {
    pub start_ts: i64,
    pub end_ts: i64,
    pub busy: bool,
}

/// This type contains a list of `EventInstance`s that are guaranteed to be
/// compatible and sorted by lowest `start_ts` first.
/// Two `EventInstance`s are compatible if they do not overlap.
#[derive(PartialEq, Debug, Default)]
pub struct CompatibleInstances {
    events: VecDeque<EventInstance>,
}

impl CompatibleInstances {
    pub fn new(mut events: Vec<EventInstance>) -> Self {
        // sort with least start_ts first
        events.sort_by(|i1, i2| i1.start_ts.cmp(&i2.start_ts));

        let mut compatible_events: VecDeque<EventInstance> = Default::default();

        for (i, instance) in events.into_iter().enumerate() {
            if i == 0 {
                compatible_events.push_back(instance);
                continue;
            }
            if let Some(merged) = EventInstance::merge(
                &instance,
                &compatible_events.get(compatible_events.len() - 1).unwrap(),
            ) {
                let len = compatible_events.len();
                compatible_events[len - 1] = merged;
            } else {
                compatible_events.push_back(instance);
            }
        }

        Self {
            events: compatible_events,
        }
    }

    pub fn remove_instances(&mut self, instances: &CompatibleInstances, skip: usize) {
        self.events = self
            .events
            .iter()
            .map(|free_instance| free_instance.remove_instances(instances, skip).inner())
            .flatten()
            .collect()
    }

    pub fn push_front(&mut self, instance: EventInstance) -> bool {
        if let Some(first_instance) = self.events.get(0) {
            // There is overlap, so cannot be added
            if first_instance.start_ts < instance.end_ts {
                return false;
            }
        }
        self.events.push_front(instance);
        true
    }

    pub fn push_back(&mut self, instance: EventInstance) -> bool {
        if !self.events.is_empty() {
            if let Some(last_instance) = self.events.get(self.events.len() - 1) {
                // There is overlap, so cannot be added
                if last_instance.end_ts > instance.start_ts {
                    return false;
                }
            }
        }
        self.events.push_back(instance);
        true
    }

    pub fn remove_all_before(&mut self, timespan: i64) {
        while let Some(e) = self.events.get_mut(0) {
            if e.start_ts >= timespan {
                break;
            }
            if e.end_ts <= timespan {
                self.events.pop_front();
            } else {
                e.start_ts = timespan;
                break;
            }
        }
    }

    pub fn remove_all_after(&mut self, timespan: i64) {
        while !self.events.is_empty() {
            let last = self.events.get_mut(self.events.len() - 1).unwrap();
            if last.end_ts <= timespan {
                break;
            }
            if last.start_ts >= timespan {
                self.events.pop_back();
            } else {
                last.end_ts = timespan;
                break;
            }
        }
    }

    pub fn extend(&mut self, instances: CompatibleInstances) {
        for instance in instances.inner() {
            self.push_back(instance);
        }
    }

    pub fn inner(self) -> VecDeque<EventInstance> {
        self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn get(&self, index: usize) -> Option<&EventInstance> {
        self.events.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl AsRef<VecDeque<EventInstance>> for CompatibleInstances {
    fn as_ref(&self) -> &VecDeque<EventInstance> {
        &self.events
    }
}

#[derive(PartialEq, Debug)]
pub enum SubtractInstanceResult {
    /// Instances does not overlap
    NoOverlap,
    /// Instances overlaps and free.start > end.start && free.end > end.end
    OverlapBeginning(CompatibleInstances),
    /// Instances overlaps and free.start < end.start && free.end < end.end
    OverlapEnd(CompatibleInstances),
    /// Instances overlaps and free.start < end.start && free.end > end.end
    Split(CompatibleInstances),
    /// Instances overlaps and free.start > énd.start && free.end < end.end
    Empty,
}

impl EventInstance {
    pub fn has_overlap(instance1: &Self, instance2: &Self) -> bool {
        instance1.start_ts <= instance2.end_ts && instance1.end_ts >= instance2.start_ts
    }

    pub fn can_merge(instance1: &Self, instance2: &Self) -> bool {
        instance1.busy == instance2.busy && Self::has_overlap(instance1, instance2)
    }

    /// Merges two `EventInstance`s into a new `EventInstance` if they overlap.
    pub fn merge(instance1: &Self, instance2: &Self) -> Option<Self> {
        if !Self::can_merge(instance1, instance2) {
            return None;
        }

        Some(Self {
            start_ts: std::cmp::min(instance1.start_ts, instance2.start_ts),
            end_ts: std::cmp::max(instance1.end_ts, instance2.end_ts),
            busy: instance1.busy,
        })
    }

    pub fn remove_instance(free_instance: &Self, instance: &Self) -> SubtractInstanceResult {
        if !Self::has_overlap(free_instance, instance) || free_instance.start_ts == instance.end_ts
        {
            return SubtractInstanceResult::NoOverlap;
        }

        if instance.start_ts <= free_instance.start_ts && instance.end_ts >= free_instance.end_ts {
            return SubtractInstanceResult::Empty;
        }

        if instance.start_ts > free_instance.start_ts && instance.end_ts < free_instance.end_ts {
            let free_instance_1 = Self {
                start_ts: free_instance.start_ts,
                end_ts: instance.start_ts,
                busy: false,
            };
            let free_instance_2 = Self {
                start_ts: instance.end_ts,
                end_ts: free_instance.end_ts,
                busy: false,
            };
            let events = CompatibleInstances::new(vec![free_instance_1, free_instance_2]);
            return SubtractInstanceResult::Split(events);
        }

        if free_instance.start_ts >= instance.start_ts {
            let e = CompatibleInstances::new(vec![Self {
                start_ts: instance.end_ts,
                end_ts: free_instance.end_ts,
                busy: false,
            }]);
            SubtractInstanceResult::OverlapBeginning(e)
        } else {
            let e = CompatibleInstances::new(vec![Self {
                start_ts: free_instance.start_ts,
                end_ts: instance.start_ts,
                busy: false,
            }]);
            SubtractInstanceResult::OverlapEnd(e)
        }
    }

    pub fn remove_instances(
        &self,
        instances: &CompatibleInstances,
        skip: usize,
    ) -> CompatibleInstances {
        let mut free_instances_without_conflict = CompatibleInstances::new(Vec::new());

        let mut conflict = false;
        for (pos, instance) in instances.as_ref().iter().skip(skip).enumerate() {
            if instance.start_ts >= self.end_ts {
                break;
            }
            let free_instances = match EventInstance::remove_instance(self, instance) {
                SubtractInstanceResult::OverlapEnd(event) => {
                    assert_eq!(event.len(), 1);
                    conflict = true;
                    Some(event)
                }
                SubtractInstanceResult::OverlapBeginning(mut event) => {
                    assert_eq!(event.len(), 1);
                    conflict = true;
                    event.remove_instances(instances, pos + 1);

                    Some(event)
                }
                SubtractInstanceResult::Split(events) => {
                    assert_eq!(events.len(), 2);
                    conflict = true;

                    let mut events = events.inner();
                    let last_event = events.pop_back().unwrap();
                    let first_event = events.pop_front().unwrap();

                    let mut events = CompatibleInstances::new(vec![last_event.clone()]);
                    events.remove_instances(instances, pos + 1);
                    events.push_front(first_event);

                    Some(events)
                }
                SubtractInstanceResult::Empty => {
                    conflict = true;
                    None
                }
                SubtractInstanceResult::NoOverlap => {
                    conflict = false;
                    None
                }
            };
            if let Some(new_free_instances) = free_instances {
                free_instances_without_conflict.extend(new_free_instances);
            }
        }
        if !conflict {
            free_instances_without_conflict.push_back(self.clone());
        }

        free_instances_without_conflict
    }
}

#[derive(Debug)]
pub struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

pub fn separate_free_busy_events(
    instances: Vec<EventInstance>,
) -> (Vec<EventInstance>, Vec<EventInstance>) {
    let mut free_instances = Vec::new();
    let mut busy_instances = Vec::new();

    for instance in instances {
        if instance.busy {
            busy_instances.push(instance);
        } else {
            free_instances.push(instance);
        }
    }

    (free_instances, busy_instances)
}

pub struct FreeBusy {
    pub free: CompatibleInstances,
    pub busy: CompatibleInstances,
}

pub fn get_free_busy(instances: Vec<EventInstance>) -> FreeBusy {
    let (free_instances, busy_instances) = separate_free_busy_events(instances);

    let mut free_instances = CompatibleInstances::new(free_instances);
    let busy_instances = CompatibleInstances::new(busy_instances);

    free_instances.remove_instances(&busy_instances, 0);

    FreeBusy {
        free: free_instances,
        busy: busy_instances,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod combining_events {

        use super::*;

        #[test]
        fn no_overlap() {
            let e1 = EventInstance {
                start_ts: 0,
                end_ts: 4,
                busy: false,
            };

            let e2 = EventInstance {
                start_ts: 5,
                end_ts: 10,
                busy: false,
            };

            let res = EventInstance::merge(&e1, &e2);
            assert!(res.is_none());
        }

        #[test]
        fn overlap_without_extending() {
            let e1 = EventInstance {
                start_ts: 1,
                end_ts: 10,
                busy: false,
            };

            let e2 = EventInstance {
                start_ts: 5,
                end_ts: 7,
                busy: false,
            };

            let res = EventInstance::merge(&e1, &e2);
            assert!(res.is_some());
            assert_eq!(res.unwrap(), e1);
        }

        #[test]
        fn overlap_with_extending() {
            let e1 = EventInstance {
                start_ts: 1,
                end_ts: 10,
                busy: false,
            };

            let e2 = EventInstance {
                start_ts: 5,
                end_ts: 15,
                busy: false,
            };

            let res = EventInstance::merge(&e1, &e2);
            assert!(res.is_some());
            assert_eq!(
                res.unwrap(),
                EventInstance {
                    start_ts: 1,
                    end_ts: 15,
                    busy: false
                }
            );
        }

        #[test]
        fn remove_busy_from_free_no_overlap() {
            let e1 = EventInstance {
                start_ts: 0,
                end_ts: 4,
                busy: false,
            };

            let e2 = EventInstance {
                start_ts: 5,
                end_ts: 10,
                busy: true,
            };

            let res = EventInstance::remove_instance(&e1, &e2);
            assert_eq!(res, SubtractInstanceResult::NoOverlap);
        }

        #[test]
        fn remove_busy_from_free_complete_overlap() {
            let e1 = EventInstance {
                start_ts: 0,
                end_ts: 4,
                busy: false,
            };

            let e2 = EventInstance {
                start_ts: 0,
                end_ts: 10,
                busy: true,
            };

            let res = EventInstance::remove_instance(&e1, &e2);
            assert_eq!(res, SubtractInstanceResult::Empty);
        }

        #[test]
        fn remove_busy_from_free_complete_partial_split_in_1() {
            let mut e1 = EventInstance {
                start_ts: 0,
                end_ts: 4,
                busy: false,
            };

            let mut e2 = EventInstance {
                start_ts: 3,
                end_ts: 10,
                busy: true,
            };

            let res = EventInstance::remove_instance(&e1, &e2);
            let expected_e = CompatibleInstances::new(vec![EventInstance {
                start_ts: 0,
                end_ts: 3,
                busy: false,
            }]);
            let expected_res = SubtractInstanceResult::OverlapEnd(expected_e);
            assert_eq!(res, expected_res);

            // Revere ordering
            e1.busy = true;
            e2.busy = false;

            let res = EventInstance::remove_instance(&e2, &e1);
            let expected_e = CompatibleInstances::new(vec![EventInstance {
                start_ts: 4,
                end_ts: 10,
                busy: false,
            }]);
            let expected_res = SubtractInstanceResult::OverlapBeginning(expected_e);
            assert_eq!(res, expected_res);
        }

        #[test]
        fn remove_busy_from_free_complete_partial_split_in_2() {
            let mut e1 = EventInstance {
                start_ts: 2,
                end_ts: 14,
                busy: false,
            };

            let mut e2 = EventInstance {
                start_ts: 3,
                end_ts: 10,
                busy: true,
            };

            let res = EventInstance::remove_instance(&e1, &e2);
            let expected_events = CompatibleInstances::new(vec![
                EventInstance {
                    start_ts: 2,
                    end_ts: 3,
                    busy: false,
                },
                EventInstance {
                    start_ts: 10,
                    end_ts: 14,
                    busy: false,
                },
            ]);
            let expected_res = SubtractInstanceResult::Split(expected_events);
            assert_eq!(res, expected_res);

            // Revere ordering is complete overlap
            e1.busy = true;
            e2.busy = false;

            let res = EventInstance::remove_instance(&e2, &e1);
            assert_eq!(res, SubtractInstanceResult::Empty);
        }
    }

    #[test]
    fn remove_busy_from_free_test_1() {
        let free1 = EventInstance {
            start_ts: 5,
            end_ts: 100,
            busy: false,
        };
        let mut free = CompatibleInstances::new(vec![free1]);

        let busy1 = EventInstance {
            start_ts: 2,
            end_ts: 40,
            busy: false,
        };
        let busy2 = EventInstance {
            start_ts: 50,
            end_ts: 70,
            busy: false,
        };
        let busy3 = EventInstance {
            start_ts: 72,
            end_ts: 75,
            busy: false,
        };
        let busy = CompatibleInstances::new(vec![busy1, busy2, busy3]);
        free.remove_instances(&busy, 0);
        let res = free.inner();
        assert_eq!(res.len(), 3);
        assert_eq!(
            res[0],
            EventInstance {
                start_ts: 40,
                end_ts: 50,
                busy: false
            }
        );
        assert_eq!(
            res[1],
            EventInstance {
                start_ts: 70,
                end_ts: 72,
                busy: false
            }
        );
        assert_eq!(
            res[2],
            EventInstance {
                start_ts: 75,
                end_ts: 100,
                busy: false
            }
        );
    }

    #[test]
    fn remove_busy_from_free_test_2() {
        let free1 = EventInstance {
            start_ts: 0,
            end_ts: 71,
            busy: false,
        };
        let free2 = EventInstance {
            start_ts: 72,
            end_ts: 74,
            busy: false,
        };
        let free3 = EventInstance {
            start_ts: 100,
            end_ts: 140,
            busy: false,
        };
        let mut free = CompatibleInstances::new(vec![free1, free2, free3]);

        let busy1 = EventInstance {
            start_ts: 2,
            end_ts: 40,
            busy: false,
        };
        let busy2 = EventInstance {
            start_ts: 50,
            end_ts: 70,
            busy: false,
        };
        let busy3 = EventInstance {
            start_ts: 72,
            end_ts: 75,
            busy: false,
        };
        let busy = CompatibleInstances::new(vec![busy1, busy2, busy3]);
        free.remove_instances(&busy, 0);

        let res = free.inner();
        assert_eq!(res.len(), 4);
        assert_eq!(
            res[0],
            EventInstance {
                start_ts: 0,
                end_ts: 2,
                busy: false
            }
        );
        assert_eq!(
            res[1],
            EventInstance {
                start_ts: 40,
                end_ts: 50,
                busy: false
            }
        );
        assert_eq!(
            res[2],
            EventInstance {
                start_ts: 70,
                end_ts: 71,
                busy: false
            }
        );
        assert_eq!(
            res[3],
            EventInstance {
                start_ts: 100,
                end_ts: 140,
                busy: false
            }
        );
    }

    #[test]
    fn compatible_events_test_1() {
        let c_events = CompatibleInstances::new(Vec::new());
        assert_eq!(c_events.as_ref().len(), 0);
    }
    #[test]
    fn compatible_events_test_2() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let c_events = CompatibleInstances::new(vec![e1.clone()]);
        let c_events = c_events.inner();
        assert_eq!(c_events.len(), 1);
        assert_eq!(c_events[0], e1);
    }
    #[test]
    fn compatible_events_test_3() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let c_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        let c_events = c_events.inner();
        assert_eq!(c_events.len(), 1);
        assert_eq!(c_events[0], e1);
    }
    #[test]
    fn compatible_events_test_4() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let c_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        let c_events = c_events.inner();
        assert_eq!(c_events.len(), 2);
        assert_eq!(c_events[0], e1);
        assert_eq!(c_events[1], e2);
    }

    #[test]
    fn compatible_events_test_5() {
        let e1 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 1,
            end_ts: 7,
            busy: false,
        };
        let e3 = EventInstance {
            start_ts: 6,
            end_ts: 14,
            busy: false,
        };
        let e4 = EventInstance {
            start_ts: 20,
            end_ts: 30,
            busy: false,
        };
        let e5 = EventInstance {
            start_ts: 24,
            end_ts: 40,
            busy: false,
        };
        let e6 = EventInstance {
            start_ts: 44,
            end_ts: 50,
            busy: false,
        };
        let c_events = CompatibleInstances::new(vec![
            e1.clone(),
            e2.clone(),
            e3.clone(),
            e4.clone(),
            e5.clone(),
            e6.clone(),
        ]);
        let c_events = c_events.inner();
        assert_eq!(c_events.len(), 3);
        assert_eq!(
            c_events[0],
            EventInstance {
                start_ts: 1,
                end_ts: 14,
                busy: false
            }
        );
        assert_eq!(
            c_events[1],
            EventInstance {
                start_ts: 20,
                end_ts: 40,
                busy: false
            }
        );
        assert_eq!(c_events[2], e6);
    }

    #[test]
    fn compatible_events_test_6() {
        let e1 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 1,
            end_ts: 7,
            busy: false,
        };
        let e3 = EventInstance {
            start_ts: 6,
            end_ts: 14,
            busy: false,
        };
        let e4 = EventInstance {
            start_ts: 20,
            end_ts: 30,
            busy: false,
        };
        let e5 = EventInstance {
            start_ts: 24,
            end_ts: 40,
            busy: false,
        };
        let c_events = CompatibleInstances::new(vec![
            e1.clone(),
            e2.clone(),
            e3.clone(),
            e4.clone(),
            e5.clone(),
        ]);
        let c_events = c_events.inner();
        assert_eq!(c_events.len(), 2);
        assert_eq!(
            c_events[0],
            EventInstance {
                start_ts: 1,
                end_ts: 14,
                busy: false
            }
        );
        assert_eq!(
            c_events[1],
            EventInstance {
                start_ts: 20,
                end_ts: 40,
                busy: false
            }
        );
    }

    #[test]
    fn another_free_busy() {
        let mut free = CompatibleInstances::new(
            (0..100)
                .map(|i| EventInstance {
                    start_ts: i * 10 + 5,
                    end_ts: i * 10 + 8,
                    busy: false,
                })
                .collect(),
        );
        let busy = CompatibleInstances::new(
            (0..200)
                .map(|i| EventInstance {
                    start_ts: i * 10 + 6,
                    end_ts: i * 10 + 7,
                    busy: false,
                })
                .collect(),
        );
        free.remove_instances(&busy, 0);
        assert_eq!(free.len(), 200);
    }

    #[test]
    fn single_event() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 10,
            busy: false,
        };

        let instances = vec![e1.clone()];
        let freebusy = get_free_busy(instances);
        assert_eq!(freebusy.free.len(), 1);
        assert_eq!(freebusy.free, CompatibleInstances::new(vec![e1]));
    }

    #[test]
    fn no_free_event() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 10,
            busy: true,
        };

        let instances = vec![e1];
        let freebusy = get_free_busy(instances).free;
        assert_eq!(freebusy.len(), 0);
    }

    #[test]
    fn simple_freebusy() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 10,
            busy: false,
        };

        let e2 = EventInstance {
            start_ts: 3,
            end_ts: 5,
            busy: true,
        };

        let instances = vec![e1, e2];
        let freebusy = get_free_busy(instances).free.inner();
        assert_eq!(freebusy.len(), 2);
        assert_eq!(
            freebusy,
            vec![
                EventInstance {
                    start_ts: 0,
                    end_ts: 3,
                    busy: false
                },
                EventInstance {
                    start_ts: 5,
                    end_ts: 10,
                    busy: false
                }
            ]
        )
    }

    fn validate_bounds(before: i64, after: i64, len: usize, events: &CompatibleInstances) {
        if len == 0 {
            assert!(events.is_empty());
            return;
        }
        assert!(!events.is_empty());
        let events = events.as_ref();
        assert!(events[0].start_ts >= before);
        assert!(events[events.len() - 1].end_ts <= after);
    }

    #[test]
    fn removes_all_before() {
        let inf = 10000;

        // First case
        let e1 = EventInstance {
            start_ts: 3,
            end_ts: 10,
            busy: false,
        };
        let mut all_events = CompatibleInstances::new(vec![e1.clone()]);
        all_events.remove_all_before(2);
        validate_bounds(2, inf, 1, &all_events);

        all_events.remove_all_before(5);
        validate_bounds(5, inf, 1, &all_events);
        all_events.remove_all_before(e1.end_ts);
        validate_bounds(5, inf, 0, &all_events);

        // Second case
        let e1 = EventInstance {
            start_ts: 3,
            end_ts: 10,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 12,
            end_ts: 20,
            busy: false,
        };
        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_before(5);
        validate_bounds(5, inf, 2, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_before(e2.start_ts + 2);
        validate_bounds(e2.start_ts + 2, inf, 1, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_before(e2.end_ts);
        validate_bounds(e2.end_ts, inf, 0, &all_events);
    }

    #[test]
    fn removes_all_after() {
        let neg_inf = -10000;

        // First case
        let e1 = EventInstance {
            start_ts: 3,
            end_ts: 10,
            busy: false,
        };
        let mut all_events = CompatibleInstances::new(vec![e1.clone()]);
        all_events.remove_all_after(e1.end_ts);
        validate_bounds(neg_inf, e1.end_ts, 1, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone()]);
        all_events.remove_all_after(5);
        validate_bounds(neg_inf, 5, 1, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone()]);
        all_events.remove_all_after(e1.start_ts);
        validate_bounds(neg_inf, e1.start_ts, 0, &all_events);

        // Second case
        let e1 = EventInstance {
            start_ts: 3,
            end_ts: 10,
            busy: false,
        };
        let e2 = EventInstance {
            start_ts: 12,
            end_ts: 20,
            busy: false,
        };
        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_after(e2.end_ts);
        validate_bounds(neg_inf, e2.end_ts, 2, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_after(e2.end_ts - 2);
        validate_bounds(neg_inf, e2.end_ts - 2, 2, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_after(e1.end_ts);
        validate_bounds(neg_inf, e1.end_ts, 1, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_after(e1.end_ts - 2);
        validate_bounds(neg_inf, e1.end_ts - 2, 1, &all_events);

        let mut all_events = CompatibleInstances::new(vec![e1.clone(), e2.clone()]);
        all_events.remove_all_after(e1.start_ts - 1);
        validate_bounds(neg_inf, e1.start_ts - 1, 0, &all_events);
    }
}
