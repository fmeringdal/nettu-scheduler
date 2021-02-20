use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInstance {
    pub start_ts: i64,
    pub end_ts: i64,
    pub busy: bool,
}

impl EventInstance {
    pub fn has_overlap(instance1: &Self, instance2: &Self) -> bool {
        instance1.start_ts <= instance2.end_ts && instance1.end_ts >= instance2.start_ts
    }

    pub fn can_merge(instance1: &Self, instance2: &Self) -> bool {
        instance1.busy == instance2.busy && Self::has_overlap(instance1, instance2)
    }

    pub fn merge(instance1: &Self, instance2: &Self) -> Option<Self> {
        if !Self::can_merge(instance1, instance2) {
            return None;
        }
        // todo: check for can merge and overlap
        Some(Self {
            start_ts: std::cmp::min(instance1.start_ts, instance2.start_ts),
            end_ts: std::cmp::max(instance1.end_ts, instance2.end_ts),
            busy: instance1.busy,
        })
    }

    pub fn remove_busy_event(free_instance: &Self, busy_instance: &Self) -> Vec<Self> {
        if !Self::has_overlap(free_instance, busy_instance) {
            return vec![free_instance.clone()];
        }

        if busy_instance.start_ts <= free_instance.start_ts
            && busy_instance.end_ts >= free_instance.end_ts
        {
            return vec![];
        }

        if busy_instance.start_ts > free_instance.start_ts
            && busy_instance.end_ts < free_instance.end_ts
        {
            let free_instance_1 = Self {
                start_ts: free_instance.start_ts,
                end_ts: busy_instance.start_ts,
                busy: false,
            };
            let free_instance_2 = Self {
                start_ts: busy_instance.end_ts,
                end_ts: free_instance.end_ts,
                busy: false,
            };
            return vec![free_instance_1, free_instance_2];
        }

        if free_instance.start_ts >= busy_instance.start_ts {
            return vec![Self {
                start_ts: busy_instance.end_ts,
                end_ts: free_instance.end_ts,
                busy: false,
            }];
        } else {
            return vec![Self {
                start_ts: free_instance.start_ts,
                end_ts: busy_instance.start_ts,
                busy: false,
            }];
        }
    }
}

fn sort_and_merge_instances(instances: &mut Vec<&mut EventInstance>) -> Vec<EventInstance> {
    // sort with least start_ts first
    instances.sort_by(|i1, i2| i1.start_ts.cmp(&i2.start_ts));

    let mut sorted: Vec<EventInstance> = vec![];

    for (i, instance) in instances.iter_mut().enumerate() {
        if i == 0 {
            sorted.push(instance.to_owned());
            continue;
        }
        if let Some(merged) = EventInstance::merge(&instance, &sorted.last().unwrap()) {
            let len = sorted.len();
            sorted[len - 1] = merged;
        } else {
            sorted.push(instance.to_owned());
        }
    }

    sorted
}

fn remove_busy_from_free_instance(
    free_instance: &EventInstance,
    busy_instances: &[EventInstance],
) -> Vec<EventInstance> {
    let mut free_instances_without_conflict = vec![];

    let mut confict = false;
    for (busy_pos, busy_instance) in busy_instances.iter().enumerate() {
        if busy_instance.start_ts >= free_instance.end_ts {
            break;
        }
        if EventInstance::has_overlap(&free_instance, &busy_instance) {
            let mut free_events = EventInstance::remove_busy_event(&free_instance, &busy_instance);

            // If remove busy events split results in a free event that has start_ts later than end_ts of busy_instance,
            // we sneed to check that free event does not conflict with any
            // of the other later busy events.
            if busy_pos < busy_instances.len() - 1
                && !free_events.is_empty()
                && free_events.last().unwrap().start_ts >= busy_instance.end_ts
            {
                let last_free_event = vec![free_events.last().unwrap().clone()];
                let last_free_events =
                    remove_busy_from_free(&last_free_event, &busy_instances[busy_pos + 1..]);
                if free_events.len() < 2 {
                    free_events = last_free_events;
                } else {
                    free_events = vec![free_events[0].clone()];
                    free_events.extend(last_free_events);
                }
            }
            free_instances_without_conflict.extend(free_events);
            confict = true;
            break;
        }
    }
    if !confict {
        free_instances_without_conflict.push(free_instance.clone());
    }

    free_instances_without_conflict
}

fn remove_busy_from_free(
    free_instances: &Vec<EventInstance>,
    busy_instances: &[EventInstance],
) -> Vec<EventInstance> {
    free_instances
        .iter()
        .map(|free_instance| remove_busy_from_free_instance(free_instance, busy_instances))
        .flatten()
        .collect()
}

// TODO: should be able to just pass in free and busy instances as params
pub fn get_free_busy(instances: &mut Vec<EventInstance>) -> Vec<EventInstance> {
    let mut free_instances = vec![];
    let mut busy_instances = vec![];

    for instance in instances.iter_mut() {
        if instance.busy {
            busy_instances.push(instance);
        } else {
            free_instances.push(instance);
        }
    }

    let free_instances = sort_and_merge_instances(&mut free_instances);
    let busy_instances = sort_and_merge_instances(&mut busy_instances);

    remove_busy_from_free(&free_instances, &busy_instances)
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

            let res = EventInstance::remove_busy_event(&e1, &e2);
            assert_eq!(res.len(), 1);
            assert_eq!(res, vec![e1]);
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

            let res = EventInstance::remove_busy_event(&e1, &e2);
            assert_eq!(res.len(), 0);
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

            let res = EventInstance::remove_busy_event(&e1, &e2);
            assert_eq!(res.len(), 1);
            assert_eq!(
                res,
                vec![EventInstance {
                    start_ts: 0,
                    end_ts: 3,
                    busy: false
                }]
            );

            // Revere ordering
            e1.busy = true;
            e2.busy = false;

            let res = EventInstance::remove_busy_event(&e2, &e1);
            assert_eq!(res.len(), 1);
            assert_eq!(
                res,
                vec![EventInstance {
                    start_ts: 4,
                    end_ts: 10,
                    busy: false
                }]
            );
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

            let res = EventInstance::remove_busy_event(&e1, &e2);
            assert_eq!(res.len(), 2);
            assert_eq!(
                res,
                vec![
                    EventInstance {
                        start_ts: 2,
                        end_ts: 3,
                        busy: false
                    },
                    EventInstance {
                        start_ts: 10,
                        end_ts: 14,
                        busy: false
                    }
                ]
            );

            // Revere ordering is complete overlap
            e1.busy = true;
            e2.busy = false;

            let res = EventInstance::remove_busy_event(&e2, &e1);
            assert_eq!(res.len(), 0);
        }
    }

    #[test]
    fn remove_busy_from_free_test_1() {
        let free1 = EventInstance {
            start_ts: 5,
            end_ts: 100,
            busy: false,
        };

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
        let res = remove_busy_from_free(&vec![free1], &[busy1, busy2, busy3]);
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
        let res = remove_busy_from_free(&vec![free1, free2, free3], &[busy1, busy2, busy3]);
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
    fn sort_and_merge_instances_test_1() {
        let res = sort_and_merge_instances(&mut vec![]);
        assert_eq!(res.len(), 0);
    }
    #[test]
    fn sort_and_merge_instances_test_2() {
        let mut e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let res = sort_and_merge_instances(&mut vec![&mut e1]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], e1);
    }
    #[test]
    fn sort_and_merge_instances_test_3() {
        let mut e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let mut e2 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let res = sort_and_merge_instances(&mut vec![&mut e1, &mut e2]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], e1);
    }
    #[test]
    fn sort_and_merge_instances_test_4() {
        let mut e1 = EventInstance {
            start_ts: 0,
            end_ts: 2,
            busy: false,
        };
        let mut e2 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let res = sort_and_merge_instances(&mut vec![&mut e1, &mut e2]);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], e1);
        assert_eq!(res[1], e2);
    }

    #[test]
    fn sort_and_merge_instances_test_5() {
        let mut e1 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let mut e2 = EventInstance {
            start_ts: 1,
            end_ts: 7,
            busy: false,
        };
        let mut e3 = EventInstance {
            start_ts: 6,
            end_ts: 14,
            busy: false,
        };
        let mut e4 = EventInstance {
            start_ts: 20,
            end_ts: 30,
            busy: false,
        };
        let mut e5 = EventInstance {
            start_ts: 24,
            end_ts: 40,
            busy: false,
        };
        let mut e6 = EventInstance {
            start_ts: 44,
            end_ts: 50,
            busy: false,
        };
        let res = sort_and_merge_instances(&mut vec![
            &mut e1, &mut e2, &mut e3, &mut e4, &mut e5, &mut e6,
        ]);
        assert_eq!(res.len(), 3);
        assert_eq!(
            res[0],
            EventInstance {
                start_ts: 1,
                end_ts: 14,
                busy: false
            }
        );
        assert_eq!(
            res[1],
            EventInstance {
                start_ts: 20,
                end_ts: 40,
                busy: false
            }
        );
        assert_eq!(res[2], e6);
    }

    #[test]
    fn sort_and_merge_instances_test_6() {
        let mut e1 = EventInstance {
            start_ts: 5,
            end_ts: 10,
            busy: false,
        };
        let mut e2 = EventInstance {
            start_ts: 1,
            end_ts: 7,
            busy: false,
        };
        let mut e3 = EventInstance {
            start_ts: 6,
            end_ts: 14,
            busy: false,
        };
        let mut e4 = EventInstance {
            start_ts: 20,
            end_ts: 30,
            busy: false,
        };
        let mut e5 = EventInstance {
            start_ts: 24,
            end_ts: 40,
            busy: false,
        };
        let res = sort_and_merge_instances(&mut vec![&mut e1, &mut e2, &mut e3, &mut e4, &mut e5]);
        assert_eq!(res.len(), 2);
        assert_eq!(
            res[0],
            EventInstance {
                start_ts: 1,
                end_ts: 14,
                busy: false
            }
        );
        assert_eq!(
            res[1],
            EventInstance {
                start_ts: 20,
                end_ts: 40,
                busy: false
            }
        );
    }

    #[test]
    fn single_event() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 10,
            busy: false,
        };

        let mut instances = vec![e1.clone()];
        let freebusy = get_free_busy(&mut instances);
        assert_eq!(freebusy.len(), 1);
        assert_eq!(freebusy, vec![e1]);
    }

    #[test]
    fn no_free_event() {
        let e1 = EventInstance {
            start_ts: 0,
            end_ts: 10,
            busy: true,
        };

        let mut instances = vec![e1];
        let freebusy = get_free_busy(&mut instances);
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

        let mut instances = vec![e1, e2];
        let freebusy = get_free_busy(&mut instances);
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
}
