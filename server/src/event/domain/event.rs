use crate::calendar::domain::calendar_view::CalendarView;

use super::event_instance::EventInstance;
use chrono::prelude::*;
use chrono_tz::Tz;
use rrule::{Frequenzy, ParsedOptions, RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RRuleOptions {
    pub freq: isize,
    pub interval: isize,
    pub count: Option<i32>,
    pub until: Option<isize>,
    pub tzid: String,
    pub wkst: isize,
    pub bysetpos: Vec<isize>,
    pub byweekday: Vec<isize>,
    pub bynweekday: Vec<Vec<isize>>,
}
#[derive(Serialize, Debug, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: Option<i64>,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<isize>,
    pub calendar_id: String,
    pub user_id: String,
}

impl CalendarEvent {
    pub fn new() -> Self {
        Self {
            id: String::from(""),
            start_ts: 0,
            duration: 0,
            end_ts: None,
            recurrence: None,
            exdates: vec![],
            busy: false,
            calendar_id: String::from(""),
            user_id: String::from(""),
        }
    }

    fn update_endtime(&mut self) {
        let opts = self.get_rrule_options();
        if (opts.count.is_some() && opts.count.unwrap() > 0) || opts.until.is_some() {
            let expand = self.expand(None);
            if let Some(last_occurence) = expand.last() {
                self.end_ts = Some(last_occurence.end_ts);
            } else {
                self.end_ts = None;
            }
        } else {
            self.end_ts = None;
        }
    }

    pub fn set_reccurrence(&mut self, reccurence: RRuleOptions, update_endtime: bool) {
        self.recurrence = Some(reccurence);
        if update_endtime {
            self.update_endtime();
        }
    }

    pub fn expand(&self, view: Option<&CalendarView>) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = self.get_rrule_options();
            println!("Opts: {:?}", rrule_options);

            let tzid = rrule_options.tzid.clone();
            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = tzid.timestamp(*exdate as i64 / 1000, 0);
                rrule_set.exdate(exdate);
            }
            let rrule = RRule::new(rrule_options);
            // println!("rr: {:?}", rrule.all());
            rrule_set.rrule(rrule);

            let instances = match view {
                Some(view) => {
                    let view = view.as_datetime(&tzid);
                    rrule_set.between(view.start, view.end, true)
                }
                None => rrule_set.all(),
            };

            instances
                .iter()
                .map(|occurence| {
                    // println!("Occurence: {:?}", occurence);
                    let start_ts = occurence.timestamp() * 1000;

                    return EventInstance {
                        start_ts,
                        end_ts: start_ts + self.duration,
                        busy: self.busy,
                    };
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

    fn freq_convert(freq: isize) -> Frequenzy {
        match freq {
            1 => Frequenzy::Yearly,
            2 => Frequenzy::Monthly,
            3 => Frequenzy::Weekly,
            4 => Frequenzy::Daily,
            _ => Frequenzy::Weekly,
        }
    }

    fn get_rrule_options(&self) -> ParsedOptions {
        let options = self.recurrence.clone().unwrap();

        let tzid: Tz = options.tzid.parse().unwrap();
        let until = match options.until {
            Some(ts) => Some(tzid.timestamp(ts as i64 / 1000, 0)),
            None => None,
        };

        let dtstart = tzid.timestamp(self.start_ts as i64 / 1000, 0);

        let count = match options.count {
            Some(c) => Some(std::cmp::max(c, 0) as u32),
            None => None,
        };

        return ParsedOptions {
            freq: Self::freq_convert(options.freq),
            count,
            bymonth: vec![],
            dtstart,
            byweekday: options
                .byweekday
                .iter()
                .map(|d| d.clone() as usize)
                .collect(),
            byhour: vec![dtstart.hour() as usize],
            bysetpos: options.bysetpos,
            byweekno: vec![],
            byminute: vec![dtstart.minute() as usize],
            bysecond: vec![dtstart.second() as usize],
            byyearday: vec![],
            bymonthday: vec![],
            bynweekday: options.bynweekday.clone(),
            bynmonthday: vec![],
            until,
            wkst: options.wkst as usize,
            tzid,
            interval: options.interval as usize,
            byeaster: None,
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> DateTime<Tz> {
        UTC.ymd(year, month, day).and_hms(hour, minute, second)
    }

    #[test]
    fn daily_calendar_event() {
        let event = CalendarEvent {
            id: String::from("dsa"),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: 4,
                interval: 1,
                tzid: UTC.to_string(),
                wkst: 0,
                until: None,
                count: Some(4),
                bynweekday: vec![],
                byweekday: vec![],
                bysetpos: vec![],
            }),
            end_ts: None,
            exdates: vec![1521317491239],
            calendar_id: String::from(""),
            user_id: String::from(""),
        };

        let oc = event.expand(None);
        assert_eq!(oc.len(), 3);
    }
}
