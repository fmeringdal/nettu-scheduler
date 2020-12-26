use super::event_instance::EventInstance;
use chrono::prelude::*;
use chrono_tz::{Tz, UTC};
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

pub struct CalendarEvent {
    pub start_ts: i64,
    pub duration: i64,
    pub end_ts: Option<i64>,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<isize>,
    pub calendar_id: String,
    pub user_id: String,
}

impl CalendarEvent {
    pub fn new() -> Self {
        Self {
            start_ts: 0,
            duration: 0,
            end_ts: None,
            recurrence: None,
            exdates: vec![],
            calendar_id: String::from(""),
            user_id: String::from(""),
        }
    }

    pub fn set_reccurrence(&mut self, reccurence: RRuleOptions) {
        self.recurrence = Some(reccurence);
        let opts = self.get_rrule_options();
        if opts.count.is_some() || opts.until.is_some() {
            let expand = self.expand();
            if let Some(last_occurence) = expand.last() {
                self.end_ts = Some(last_occurence.end_ts);
            } else {
                self.end_ts = None;
            }
        } else {
            self.end_ts = None;
        }
    }

    pub fn expand(&self) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = self.get_rrule_options();
            println!("Opts: {:?}", rrule_options);

            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = rrule_options.tzid.timestamp(*exdate as i64 / 1000, 0);
                rrule_set.exdate(exdate);
            }

            let mut rrule = RRule::new(rrule_options);
            println!("rr: {:?}", rrule.all());
            rrule_set.rrule(rrule);

            rrule_set
                .all()
                .iter()
                .map(|occurence| {
                    println!("Occurence: {:?}", occurence);
                    let start_ts = occurence.timestamp();

                    return EventInstance {
                        start_ts,
                        end_ts: start_ts + self.duration,
                        busy: false,
                    };
                })
                .collect()
        } else {
            vec![EventInstance {
                start_ts: self.start_ts,
                end_ts: self.start_ts + self.duration,
                busy: false,
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
            Some(c) => Some(c as u32),
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
            start_ts: 1521317491239,
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

        let oc = event.expand();
        assert_eq!(oc.len(), 3);
    }
}
