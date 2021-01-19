use std::collections::HashMap;

use crate::{calendar::domain::{date, CalendarView}, event::domain::event_instance::EventInstance, shared::entity::Entity};
use chrono::prelude::*;
use chrono_tz::Tz;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: String,
    pub rules: Vec<ScheduleRule>,
    pub timezone: String,
}

impl Entity for Schedule {
    fn id(&self) -> String {
        self.id.clone()
    }
} 

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScheduleRuleVariant {
    WDay(Weekday),
    Date(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Time {
    hours: u32,
    minutes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleRuleInterval {
    start: Time,
    end: Time,
}

impl ScheduleRuleInterval {
    pub fn to_event(&self, day: &Day, tzid: &Tz) -> EventInstance {
        EventInstance {
            busy: false,
            start_ts: tzid
                .ymd(day.year, day.month, day.day)
                .and_hms(self.start.hours, self.start.minutes, 0)
                .timestamp_millis(),
            end_ts: tzid
                .ymd(day.year, day.month, day.day)
                .and_hms(self.end.hours, self.end.minutes, 0)
                .timestamp_millis(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleRule {
    pub variant: ScheduleRuleVariant,
    pub intervals: Vec<ScheduleRuleInterval>,
}

#[derive(Debug, PartialEq)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Day {
    pub fn inc(&mut self) {
        if self.day == date::get_month_length(self.year, self.month) {
            self.day = 1;
            if self.month == 12 {
                self.month = 1;
                self.year += 1;
            } else {
                self.month += 1;
            }
        } else {
            self.day += 1;
        }
    }

    pub fn weekday(&self, tzid: &Tz) -> Weekday {
        let date = tzid.ymd(self.year, self.month, self.day);
        date.weekday()
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

impl std::cmp::PartialOrd for Day {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.year.cmp(&other.year) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };
        match self.month.cmp(&other.month) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };
        Some(self.day.cmp(&other.day))
    }
}

impl Schedule {
    pub fn freebusy(&self, view: &CalendarView) -> Vec<EventInstance> {
        let tz: Tz = self.timezone.parse().unwrap();
        let start = tz.timestamp_millis(view.get_start());
        let end = tz.timestamp_millis(view.get_end());

        let mut date_lookup = HashMap::new();
        let mut weekday_lookup = HashMap::new();
        for rule in &self.rules {
            match &rule.variant {
                ScheduleRuleVariant::Date(date) => {
                    date_lookup.insert(date, &rule.intervals);
                }
                ScheduleRuleVariant::WDay(wkay) => {
                    weekday_lookup.insert(wkay, &rule.intervals);
                }
            }
        }

        let mut free_instances = vec![];

        let mut day_cursor = Day {
            year: start.year(),
            month: start.month(),
            day: start.day(),
        };
        let last_day = Day {
            year: end.year(),
            month: end.month(),
            day: end.day(),
        };
        while day_cursor <= last_day {
            let day_str = day_cursor.to_string();

            let intervals = match date_lookup.get(&day_str) {
                Some(intervals) => Some(intervals),
                None => {
                    // check if weekday rule exists
                    let weekday = day_cursor.weekday(&tz);
                    weekday_lookup.get(&weekday)
                }
            };
            if let Some(intervals) = intervals {
                free_instances.extend(
                    intervals
                        .into_iter()
                        .map(|interval| interval.to_event(&day_cursor, &tz)),
                );
            }
            day_cursor.inc();
        }
        std::mem::drop(date_lookup);

        free_instances
    }
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn day_sanity_tests(){
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 1
        };
        day.inc();
        assert_eq!(day, Day {
            year: 2021,
            month: 1,
            day: 2
        });
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 31
        };
        day.inc();
        assert_eq!(day, Day {
            year: 2021,
            month: 2,
            day: 1
        });
        let mut day = Day {
            year: 2021,
            month: 12,
            day: 31
        };
        day.inc();
        assert_eq!(day, Day {
            year: 2022,
            month: 1,
            day: 1
        });
        for _ in 0..365 {
            day.inc();    
        }
        assert_eq!(day, Day {
            year: 2023,
            month: 1,
            day: 1
        });
    }

    #[test]
    fn it_computes_freebusy_for_schedule() {
        let schedule = Schedule {
            id: "0".into(),
            timezone: "UTC".into(),
            rules: vec![
                ScheduleRule {
                    variant: ScheduleRuleVariant::WDay(Weekday::Mon),
                    intervals: vec![
                        ScheduleRuleInterval {
                            start: Time {
                                hours: 8,
                                minutes: 0
                            },
                            end: Time {
                                hours: 10,
                                minutes: 30
                            }
                        }
                    ]
                },
                ScheduleRule {
                    variant: ScheduleRuleVariant::Date("1970-1-12".into()),
                    intervals: vec![
                        ScheduleRuleInterval {
                            start: Time {
                                hours: 9,
                                minutes: 0
                            },
                            end: Time {
                                hours: 12,
                                minutes: 30
                            }
                        }
                    ]
                }
            ]
        };

        let view = CalendarView::create(0, 1000*60*60*24*30).unwrap();
        let freebusy = schedule.freebusy(&view);

        assert_eq!(freebusy.len(), 4);
        assert_eq!(freebusy[0], EventInstance { start_ts: 374400000, end_ts: 383400000, busy: false });
        // Check that Date variant ovverides wday variant
        assert_eq!(freebusy[1], EventInstance { start_ts: 982800000, end_ts: 995400000, busy: false });
        assert_eq!(freebusy[2], EventInstance { start_ts: 1584000000, end_ts: 1593000000, busy: false });
        assert_eq!(freebusy[3], EventInstance { start_ts: 2188800000, end_ts: 2197800000, busy: false });
    }
}