use crate::event_instance::EventInstance;
use chrono::prelude::*;
use chrono_tz::{Tz, UTC};
use rrule::{Frequenzy, ParsedOptions, RRule, RRuleSet};

#[derive(Clone, Debug)]
struct RRuleOptions {
    pub freq: Frequenzy,
    pub interval: usize,
    pub count: Option<u32>,
    pub until: Option<usize>,
    pub tzid: Tz,
    pub wkst: usize,
    pub bysetpos: Vec<isize>,
    pub byweekday: Vec<usize>,
    pub bynweekday: Vec<Vec<isize>>,
}

struct CalendarEvent {
    start_ts: isize,
    duration: isize,
    end_ts: Option<isize>,
    recurrence: Option<RRuleOptions>,
    exdates: Vec<isize>,
}

impl CalendarEvent {
    pub fn new() -> Self {
        Self {
            start_ts: 0,
            duration: 0,
            end_ts: None,
            recurrence: None,
            exdates: vec![],
        }
    }

    pub fn expand(&self) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = self.get_rrule_options();
            println!("Opts: {:?}", rrule_options);
            let mut rrule = RRule::new(rrule_options);
            println!("rr: {:?}", rrule.all());
            let mut rrule_set = RRuleSet::new();
            rrule_set.rrule(rrule);
            for exdate in &self.exdates {
                let exdate = self
                    .recurrence
                    .clone()
                    .unwrap()
                    .tzid
                    .timestamp(*exdate as i64 / 1000, 0);
                rrule_set.exdate(exdate);
            }

            rrule_set
                .all()
                .iter()
                .map(|occurence| {
                    println!("Occurence: {:?}", occurence);
                    let start_ts = occurence.timestamp() as isize;

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

    fn get_rrule_options(&self) -> ParsedOptions {
        let options = self.recurrence.clone().unwrap();

        let until = match options.until {
            Some(ts) => Some(options.tzid.timestamp(ts as i64 / 1000, 0)),
            None => None,
        };

        let dtstart = options.tzid.timestamp(self.start_ts as i64 / 1000, 0);
        println!("Dtstart: {:?}", dtstart);

        return ParsedOptions {
            freq: options.freq.clone(),
            count: options.count,
            bymonth: vec![],
            dtstart,
            byweekday: options.byweekday.clone(), // ! todo
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
            wkst: options.wkst,
            tzid: options.tzid,
            interval: options.interval,
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
                freq: Frequenzy::Daily,
                interval: 1,
                tzid: UTC,
                wkst: 0,
                until: None,
                count: Some(4),
                bynweekday: vec![],
                byweekday: vec![],
                bysetpos: vec![],
            }),
            end_ts: None,
            exdates: vec![1521317491239],
        };

        let oc = event.expand();
        assert_eq!(oc.len(), 3);
    }
}
