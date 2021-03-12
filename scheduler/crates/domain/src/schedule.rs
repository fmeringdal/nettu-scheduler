use crate::{
    date,
    event_instance::EventInstance,
    shared::entity::{Entity, ID},
    timespan::TimeSpan,
    CompatibleInstances,
};
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub rules: Vec<ScheduleRule>,
    pub timezone: Tz,
}

impl Schedule {
    pub fn new(user_id: ID, account_id: ID, timezone: &Tz) -> Self {
        Self {
            id: Default::default(),
            user_id,
            account_id,
            rules: ScheduleRule::default_rules(),
            timezone: timezone.to_owned(),
        }
    }

    pub fn set_rules(&mut self, rules: &Vec<ScheduleRule>) {
        let now = Utc::now();
        let min_date = self.timezone.ymd(now.year(), now.month(), now.day()) - Duration::days(2);
        let max_date = self.timezone.ymd(min_date.year() + 5, 1, 1);
        let allowed_rules = rules
            .clone()
            .into_iter()
            .filter(|r| match &r.variant {
                ScheduleRuleVariant::Date(datestr) => match datestr.parse::<Day>() {
                    Ok(day) => {
                        let date = day.date(&self.timezone);
                        date > min_date && date < max_date
                    }
                    Err(_) => false,
                },
                _ => true,
            })
            .map(|mut r| {
                r.parse_intervals();
                r
            })
            .collect();
        self.rules = allowed_rules;
    }
}

impl Entity for Schedule {
    fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum ScheduleRuleVariant {
    WDay(Weekday),
    Date(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct Time {
    pub hours: u32,
    pub minutes: u32,
}

impl std::cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.hours.cmp(&other.hours) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };

        Some(self.minutes.cmp(&other.minutes))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

impl ScheduleRule {
    fn default_rules() -> Vec<Self> {
        let mut weekly_rules = vec![];
        let weekdays = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];
        for wday in weekdays {
            weekly_rules.push(ScheduleRule {
                variant: ScheduleRuleVariant::WDay(wday),
                intervals: vec![ScheduleRuleInterval {
                    start: Time {
                        hours: 9,
                        minutes: 0,
                    },
                    end: Time {
                        hours: 17,
                        minutes: 30,
                    },
                }],
            });
        }
        weekly_rules
    }

    fn parse_intervals(&mut self) {
        if self.intervals.len() > 10 {
            self.intervals.splice(10.., vec![]);
        }
        // earliest start first
        self.intervals
            .sort_by(|i1, i2| i1.start.partial_cmp(&i2.start).unwrap());

        self.intervals
            .retain(|interval| interval.start <= interval.end);

        let mut remove_intervals = HashMap::new();

        for i in 0..self.intervals.len() {
            if remove_intervals.get(&i).is_some() {
                continue;
            }
            for j in (i + 1)..self.intervals.len() {
                if remove_intervals.get(&j).is_some() {
                    continue;
                }
                if self.intervals[j].start == self.intervals[i].start
                    || self.intervals[j].start <= self.intervals[i].end
                {
                    if self.intervals[j].end > self.intervals[i].end {
                        self.intervals[i].end = self.intervals[j].end.clone();
                    }
                    remove_intervals.insert(j, true);
                }
            }
        }

        let mut remove_intervals = remove_intervals
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>();
        // largest index first
        remove_intervals.sort_by(|i1, i2| i2.partial_cmp(&i1).unwrap());
        for index in remove_intervals {
            self.intervals.remove(index);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl FromStr for Day {
    type Err = ();

    fn from_str(datestr: &str) -> Result<Self, Self::Err> {
        date::is_valid_date(datestr)
            .map(|(year, month, day)| Day { year, month, day })
            .map_err(|_| ())
    }
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
        self.date(tzid).weekday()
    }

    pub fn date(&self, tzid: &Tz) -> Date<Tz> {
        tzid.ymd(self.year, self.month, self.day)
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
    pub fn freebusy(&self, timespan: &TimeSpan) -> CompatibleInstances {
        let start = self.timezone.timestamp_millis(timespan.start());
        let end = self.timezone.timestamp_millis(timespan.end());

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

        let mut free_instances = CompatibleInstances::new(vec![]);

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
                    let weekday = day_cursor.weekday(&self.timezone);
                    weekday_lookup.get(&weekday)
                }
            };
            if let Some(intervals) = intervals {
                for interval in intervals.iter() {
                    let event = interval.to_event(&day_cursor, &self.timezone);
                    free_instances.push_back(event);
                }
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
    fn day_sanity_tests() {
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 1,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2021,
                month: 1,
                day: 2
            }
        );
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 31,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2021,
                month: 2,
                day: 1
            }
        );
        let mut day = Day {
            year: 2021,
            month: 12,
            day: 31,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2022,
                month: 1,
                day: 1
            }
        );
        for _ in 0..365 {
            day.inc();
        }
        assert_eq!(
            day,
            Day {
                year: 2023,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn it_computes_freebusy_for_schedule() {
        let schedule = Schedule {
            id: Default::default(),
            user_id: Default::default(),
            account_id: Default::default(),
            timezone: chrono_tz::UTC,
            rules: vec![
                ScheduleRule {
                    variant: ScheduleRuleVariant::WDay(Weekday::Mon),
                    intervals: vec![ScheduleRuleInterval {
                        start: Time {
                            hours: 8,
                            minutes: 0,
                        },
                        end: Time {
                            hours: 10,
                            minutes: 30,
                        },
                    }],
                },
                ScheduleRule {
                    variant: ScheduleRuleVariant::Date("1970-1-12".into()),
                    intervals: vec![ScheduleRuleInterval {
                        start: Time {
                            hours: 9,
                            minutes: 0,
                        },
                        end: Time {
                            hours: 12,
                            minutes: 30,
                        },
                    }],
                },
            ],
        };

        let timespan = TimeSpan::new(0, 1000 * 60 * 60 * 24 * 30);
        let freebusy = schedule.freebusy(&timespan).inner();

        assert_eq!(freebusy.len(), 4);
        assert_eq!(
            freebusy[0],
            EventInstance {
                start_ts: 374400000,
                end_ts: 383400000,
                busy: false
            }
        );
        // Check that Date variant ovverides wday variant
        assert_eq!(
            freebusy[1],
            EventInstance {
                start_ts: 982800000,
                end_ts: 995400000,
                busy: false
            }
        );
        assert_eq!(
            freebusy[2],
            EventInstance {
                start_ts: 1584000000,
                end_ts: 1593000000,
                busy: false
            }
        );
        assert_eq!(
            freebusy[3],
            EventInstance {
                start_ts: 2188800000,
                end_ts: 2197800000,
                busy: false
            }
        );
    }

    #[test]
    fn it_parses_intervals_for_rule() {
        let interval1 = ScheduleRuleInterval {
            start: Time {
                hours: 8,
                minutes: 30,
            },
            end: Time {
                hours: 9,
                minutes: 0,
            },
        };
        let interval2 = ScheduleRuleInterval {
            start: Time {
                hours: 10,
                minutes: 30,
            },
            end: Time {
                hours: 12,
                minutes: 30,
            },
        };
        let interval3 = ScheduleRuleInterval {
            start: Time {
                hours: 20,
                minutes: 30,
            },
            end: Time {
                hours: 21,
                minutes: 0,
            },
        };
        let interval4 = ScheduleRuleInterval {
            start: Time {
                hours: 20,
                minutes: 45,
            },
            end: Time {
                hours: 21,
                minutes: 50,
            },
        };
        let interval5 = ScheduleRuleInterval {
            start: Time {
                hours: 21,
                minutes: 50,
            },
            end: Time {
                hours: 22,
                minutes: 50,
            },
        };

        let mut rule = ScheduleRule {
            variant: ScheduleRuleVariant::WDay(Weekday::Mon),
            intervals: vec![
                interval2.clone(),
                interval1.clone(),
                interval3.clone(),
                interval4.clone(),
                interval5.clone(),
            ],
        };

        rule.parse_intervals();
        assert_eq!(rule.intervals.len(), 3);
        assert_eq!(
            rule.intervals,
            vec![
                interval1,
                interval2,
                ScheduleRuleInterval {
                    start: Time {
                        hours: interval3.start.hours,
                        minutes: interval3.start.minutes
                    },
                    end: Time {
                        hours: interval5.end.hours,
                        minutes: interval5.end.minutes
                    }
                },
            ]
        );
    }
}
