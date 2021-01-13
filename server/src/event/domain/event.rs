use crate::{calendar::domain::calendar_view::CalendarView, shared::entity::Entity};

use super::event_instance::EventInstance;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use rrule::{Frequenzy, ParsedOptions, RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RRuleFrequenzy {
    Yearly,
    Monthly,
    Weekly,
    Daily,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RRuleOptions {
    pub freq: RRuleFrequenzy,
    pub interval: isize,
    pub count: Option<i32>,
    pub until: Option<isize>,
    pub tzid: String,
    pub wkst: isize,
    pub bysetpos: Option<Vec<isize>>,
    pub byweekday: Option<Vec<isize>>,
    pub bynweekday: Option<Vec<Vec<isize>>>,
}

impl Default for RRuleOptions {
    fn default() -> Self {
        Self {
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            bynweekday: None,
            byweekday: None,
            bysetpos: None,
            count: None,
            tzid: Utc.to_string(),
            until: None,
            wkst: 0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
    pub calendar_id: String,
    pub user_id: String,
    pub account_id: String,
    pub reminder: Option<CalendarEventReminder>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CalendarEventReminder {
    pub minutes_before: i64,
}

fn is_none_or_empty<T>(v: &Option<Vec<T>>) -> bool {
    match v {
        Some(v) if !v.is_empty() => false,
        _ => true,
    }
}

impl CalendarEvent {
    fn validate_recurrence(start_ts: i64, recurrence: &RRuleOptions) -> bool {
        if let Some(count) = recurrence.count {
            if count > 740 || count < 1 {
                return false;
            }
        }
        let two_years_in_millis = 1000 * 60 * 60 * 24 * 366 * 2;
        if let Some(until) = recurrence.until.map(|val| val as i64) {
            if until < start_ts || until - start_ts > two_years_in_millis {
                return false;
            }
        }

        if !is_none_or_empty(&recurrence.bynweekday) && !is_none_or_empty(&recurrence.byweekday) {
            return false;
        }

        if !is_none_or_empty(&recurrence.bysetpos) && is_none_or_empty(&recurrence.byweekday) {
            return false;
        }

        if !is_none_or_empty(&recurrence.bysetpos) && !is_none_or_empty(&recurrence.bynweekday) {
            return false;
        }

        if (!is_none_or_empty(&recurrence.bysetpos) || !is_none_or_empty(&recurrence.bynweekday))
            && recurrence.freq != RRuleFrequenzy::Monthly
        {
            return false;
        }

        true
    }

    fn update_endtime(&mut self) -> bool {
        let opts = match self.get_rrule_options() {
            Ok(opts) => opts,
            Err(_) => return false,
        };
        if (opts.count.is_some() && opts.count.unwrap() > 0) || opts.until.is_some() {
            let expand = self.expand(None);
            self.end_ts = expand.last().unwrap().end_ts;
        } else {
            self.end_ts = Self::get_max_timestamp();
        }
        true
    }

    pub fn set_recurrence(&mut self, reccurence: RRuleOptions, update_endtime: bool) -> bool {
        let valid_recurrence = Self::validate_recurrence(self.start_ts, &reccurence);
        if !valid_recurrence {
            return false;
        }

        self.recurrence = Some(reccurence);
        if update_endtime {
            return self.update_endtime();
        }
        true
    }

    pub fn get_max_timestamp() -> i64 {
        5609882500905 // Mon Oct 09 2147 06:41:40 GMT+0200 (Central European Summer Time)
    }

    pub fn expand(&self, view: Option<&CalendarView>) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = match self.get_rrule_options() {
                Ok(opts) => opts,
                Err(_) => return Default::default(),
            };

            let tzid = rrule_options.tzid;
            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = tzid.timestamp_millis(*exdate);
                rrule_set.exdate(exdate);
            }
            let rrule = RRule::new(rrule_options);
            rrule_set.rrule(rrule);

            let instances = match view {
                Some(view) => {
                    let view = view.as_datetime(&tzid);

                    // Also take the duration of events into consideration as the rrule library
                    // does not support duration on events.
                    let end = view.end - Duration::milliseconds(self.duration);

                    rrule_set.between(view.start, end, true)
                }
                None => rrule_set.all(),
            };

            instances
                .iter()
                .map(|occurence| {
                    let start_ts = occurence.timestamp_millis();

                    EventInstance {
                        start_ts,
                        end_ts: start_ts + self.duration,
                        busy: self.busy,
                    }
                })
                .collect()
        } else {
            vec![EventInstance {
                start_ts: self.start_ts,
                end_ts: self.start_ts + self.duration,
                busy: self.busy,
            }]
        }
    }

    fn freq_convert(freq: &RRuleFrequenzy) -> Frequenzy {
        match freq {
            RRuleFrequenzy::Yearly => Frequenzy::Yearly,
            RRuleFrequenzy::Monthly => Frequenzy::Monthly,
            RRuleFrequenzy::Weekly => Frequenzy::Weekly,
            RRuleFrequenzy::Daily => Frequenzy::Daily,
        }
    }

    fn get_rrule_options(&self) -> anyhow::Result<ParsedOptions> {
        let options = self.recurrence.clone().unwrap();

        let tzid: Tz = match options.tzid.parse() {
            Ok(tzid) => tzid,
            Err(_) => return Err(anyhow::Error::msg("Invalid tzid")),
        };
        let until = match options.until {
            Some(ts) => Some(tzid.timestamp(ts as i64 / 1000, 0)),
            None => None,
        };

        let dtstart = tzid.timestamp(self.start_ts / 1000, 0);

        let count = match options.count {
            Some(c) => Some(std::cmp::max(c, 0) as u32),
            None => None,
        };

        Ok(ParsedOptions {
            freq: Self::freq_convert(&options.freq),
            count,
            bymonth: vec![],
            dtstart,
            byweekday: options
                .byweekday
                .unwrap_or_default()
                .iter()
                .map(|d| *d as usize)
                .collect(),
            byhour: vec![dtstart.hour() as usize],
            bysetpos: options.bysetpos.unwrap_or_default(),
            byweekno: vec![],
            byminute: vec![dtstart.minute() as usize],
            bysecond: vec![dtstart.second() as usize],
            byyearday: vec![],
            bymonthday: vec![],
            bynweekday: options.bynweekday.clone().unwrap_or_default(),
            bynmonthday: vec![],
            until,
            wkst: options.wkst as usize,
            tzid,
            interval: options.interval as usize,
            byeaster: None,
        })
    }
}

impl Entity for CalendarEvent {
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono_tz::UTC;

    #[test]
    fn daily_calendar_event() {
        let event = CalendarEvent {
            id: String::from("dsa"),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequenzy::Daily,
                interval: 1,
                tzid: UTC.to_string(),
                wkst: 0,
                until: None,
                count: Some(4),
                bynweekday: None,
                byweekday: None,
                bysetpos: None,
            }),
            end_ts: 2521317491239,
            exdates: vec![1521317491239],
            calendar_id: String::from(""),
            user_id: String::from(""),
            account_id: String::from(""),
            reminder: None,
        };

        let oc = event.expand(None);
        assert_eq!(oc.len(), 3);
    }

    #[test]
    fn calendar_event_without_recurrence() {
        let event = CalendarEvent {
            id: String::from("dsa"),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: None,
            end_ts: 2521317491239,
            exdates: vec![1521317491239],
            calendar_id: String::from(""),
            user_id: String::from(""),
            account_id: String::from(""),
            reminder: None,
        };

        let oc = event.expand(None);
        assert_eq!(oc.len(), 1);
    }

    #[test]
    fn rejects_event_with_invalid_recurrence() {
        let mut invalid_rrules = vec![];
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.ymd(2150, 1, 1).and_hms(0, 0, 0).timestamp_millis() as isize), // too big until
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            tzid: "safsafsa".into(), // Invalid tzid
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Both byweekday and bynweekday is set
            byweekday: Some(vec![1]),
            bynweekday: Some(vec![vec![1, 1]]),
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Both bysetpos and bynweekday is set
            bysetpos: Some(vec![1]),
            bynweekday: Some(vec![vec![1, 1]]),
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Only bysetpos and no by*
            bysetpos: Some(vec![1]),
            freq: RRuleFrequenzy::Monthly,
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Bysetpos and freq=monthly
            bysetpos: Some(vec![1]),
            byweekday: Some(vec![1]),
            freq: RRuleFrequenzy::Daily,
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut event = CalendarEvent {
                id: String::from("dsa"),
                start_ts: 1521317491239,
                busy: false,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
                exdates: vec![],
                calendar_id: String::from(""),
                user_id: String::from(""),
            account_id: String::from(""),
                recurrence: None,
                reminder: None,
            };

            assert!(!event.set_recurrence(rrule, true));
        }
    }

    #[test]
    fn allows_event_with_valid_recurrence() {
        let mut valid_rrules = vec![];
        let start_ts = 1521317491239;
        valid_rrules.push(Default::default());
        valid_rrules.push(RRuleOptions {
            count: Some(100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            until: Some(start_ts + 1000 * 60 * 60 * 24 * 100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![1]),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            bynweekday: Some(vec![vec![1, 1]]),
            freq: RRuleFrequenzy::Monthly,
            ..Default::default()
        });
        for rrule in valid_rrules {
            let mut event = CalendarEvent {
                id: String::from("dsa"),
                start_ts: start_ts as i64,
                busy: false,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
                exdates: vec![],
                calendar_id: String::from(""),
            account_id: String::from(""),
                user_id: String::from(""),
                recurrence: None,
                reminder: None,
            };

            assert!(event.set_recurrence(rrule, true));
        }
    }
}
