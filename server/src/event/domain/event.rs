use crate::{calendar::domain::calendar_view::CalendarView, shared::entity::Entity};

use super::event_instance::EventInstance;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use rrule::{Frequenzy, NWeekday, Options, ParsedOptions, RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

fn is_none_or_empty<T>(v: &Option<Vec<T>>) -> bool {
    match v {
        Some(v) if !v.is_empty() => false,
        _ => true,
    }
}

impl CalendarEvent {
    fn validate_recurrence(start_ts: i64, recurrence: &RRuleOptions) -> Result<(), ()> {
        if let Some(count) = recurrence.count {
            if count > 740 || count < 1 {
                return Err(());
            }
        }
        let two_years_in_millis = 1000 * 60 * 60 * 24 * 366 * 2;
        if let Some(until) = recurrence.until.map(|val| val as i64) {
            if until < start_ts || until - start_ts > two_years_in_millis {
                return Err(());
            }
        }

        if !is_none_or_empty(&recurrence.bysetpos) && is_none_or_empty(&recurrence.byweekday) {
            return Err(());
        }

        if !is_none_or_empty(&recurrence.bysetpos) && !is_none_or_empty(&recurrence.bynweekday) {
            return Err(());
        }

        Ok(())
    }

    fn update_endtime(&mut self) {
        let opts = self.get_rrule_options();
        if (opts.count.is_some() && opts.count.unwrap() > 0) || opts.until.is_some() {
            let expand = self.expand(None);
            self.end_ts = expand.last().unwrap().end_ts;
        } else {
            self.end_ts = Self::get_max_timestamp();
        }
    }

    pub fn set_reccurrence(
        &mut self,
        reccurence: RRuleOptions,
        update_endtime: bool,
    ) -> Result<(), ()> {
        Self::validate_recurrence(self.start_ts, &reccurence)?;
        self.recurrence = Some(reccurence);
        if update_endtime {
            self.update_endtime();
        }
        Ok(())
    }

    pub fn get_max_timestamp() -> i64 {
        5609882500905 // Mon Oct 09 2147 06:41:40 GMT+0200 (Central European Summer Time)
    }

    pub fn expand(&self, view: Option<&CalendarView>) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = self.get_rrule_options();

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

    fn get_rrule_options(&self) -> ParsedOptions {
        let options = self.recurrence.clone().unwrap();

        let tzid: Tz = options.tzid.parse().unwrap();
        let until = match options.until {
            Some(ts) => Some(tzid.timestamp(ts as i64 / 1000, 0)),
            None => None,
        };

        let dtstart = tzid.timestamp(self.start_ts / 1000, 0);

        let count = match options.count {
            Some(c) => Some(std::cmp::max(c, 0) as u32),
            None => None,
        };

        return ParsedOptions {
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
        };
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
        };

        let oc = event.expand(None);
        assert_eq!(oc.len(), 3);
    }
}
