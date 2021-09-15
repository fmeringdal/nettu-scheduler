use crate::CalendarSettings;
use chrono::prelude::*;
use rrule::{Frequenzy, ParsedOptions};
use serde::{de::Visitor, Deserialize, Serialize};
use std::cmp::Ordering;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RRuleFrequency {
    Yearly,
    Monthly,
    Weekly,
    Daily,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RRuleOptions {
    pub freq: RRuleFrequency,
    pub interval: isize,
    pub count: Option<i32>,
    pub until: Option<isize>,
    pub bysetpos: Option<Vec<isize>>,
    pub byweekday: Option<Vec<WeekDay>>,
    pub bymonthday: Option<Vec<isize>>,
    pub bymonth: Option<Vec<Month>>,
    pub byyearday: Option<Vec<isize>>,
    pub byweekno: Option<Vec<isize>>,
}

fn freq_convert(freq: &RRuleFrequency) -> Frequenzy {
    match freq {
        RRuleFrequency::Yearly => Frequenzy::Yearly,
        RRuleFrequency::Monthly => Frequenzy::Monthly,
        RRuleFrequency::Weekly => Frequenzy::Weekly,
        RRuleFrequency::Daily => Frequenzy::Daily,
    }
}

fn is_none_or_empty<T>(v: &Option<Vec<T>>) -> bool {
    !matches!(v, Some(v) if !v.is_empty())
}

impl RRuleOptions {
    pub fn is_valid(&self, start_ts: i64) -> bool {
        if let Some(count) = self.count {
            if !(1..740).contains(&count) {
                return false;
            }
        }
        let two_years_in_millis = 1000 * 60 * 60 * 24 * 366 * 2;
        if let Some(until) = self.until.map(|val| val as i64) {
            if until < start_ts || until - start_ts > two_years_in_millis {
                return false;
            }
        }

        if let Some(bysetpos) = &self.bysetpos {
            // Check that bysetpos is used with some other by* rule
            if !bysetpos.is_empty()
                && is_none_or_empty(&self.byweekday)
                && is_none_or_empty(&self.byweekno)
                && is_none_or_empty(&self.bymonth)
                && is_none_or_empty(&self.bymonthday)
                && is_none_or_empty(&self.byyearday)
            {
                // No other by* rule was specified
                return false;
            }
        }

        true
    }

    pub fn get_parsed_options(
        self,
        start_ts: i64,
        calendar_settings: &CalendarSettings,
    ) -> ParsedOptions {
        let timezone = calendar_settings.timezone;

        let until = self.until.map(|ts| timezone.timestamp(ts as i64 / 1000, 0));

        let dtstart = timezone.timestamp(start_ts / 1000, 0);

        let count = self.count.map(|c| std::cmp::max(c, 0) as u32);

        let mut byweekday = Vec::new();
        let mut bynweekday = Vec::new();
        if let Some(opts_byweekday) = self.byweekday {
            for wday in opts_byweekday {
                match wday.nth() {
                    None => byweekday.push(wday.weekday() as usize),
                    Some(n) => {
                        bynweekday.push(vec![wday.weekday() as isize, n]);
                    }
                }
            }
        }

        let mut bymonthday = Vec::new();
        let mut bynmonthday = Vec::new();
        if let Some(opts_bymonthday) = self.bymonthday {
            for monthday in opts_bymonthday {
                match monthday.cmp(&0) {
                    Ordering::Greater => bymonthday.push(monthday),
                    Ordering::Less => bynmonthday.push(monthday),
                    Ordering::Equal => {}
                }
            }
        }

        ParsedOptions {
            freq: freq_convert(&self.freq),
            count,
            dtstart,
            bymonth: self
                .bymonth
                .unwrap_or_default()
                .into_iter()
                .map(|m| m as usize)
                .collect::<Vec<_>>(),
            bymonthday,
            bynmonthday,
            byweekday,
            bynweekday,
            byyearday: self.byyearday.unwrap_or_default(),
            bysetpos: self.bysetpos.unwrap_or_default(),
            byweekno: self.byweekno.unwrap_or_default(),
            byhour: vec![dtstart.hour() as usize],
            byminute: vec![dtstart.minute() as usize],
            bysecond: vec![dtstart.second() as usize],
            until,
            wkst: calendar_settings.week_start as usize,
            tzid: timezone,
            interval: self.interval as usize,
            byeaster: None,
        }
    }
}

impl Default for RRuleOptions {
    fn default() -> Self {
        Self {
            freq: RRuleFrequency::Daily,
            interval: 1,
            byweekday: None,
            bysetpos: None,
            count: None,
            until: None,
            bymonthday: None,
            bymonth: None,
            byyearday: None,
            byweekno: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeekDay {
    n: Option<isize>,
    weekday: Weekday,
}

impl WeekDay {
    fn create(weekday: Weekday, n: Option<isize>) -> Option<Self> {
        if let Some(n) = n {
            if !Self::is_valid_n(n) {
                return None;
            }
        }
        Some(Self { n, weekday })
    }

    pub fn nth(&self) -> Option<isize> {
        self.n
    }
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    pub fn new(weekday: Weekday) -> Option<Self> {
        Self::create(weekday, None)
    }

    pub fn new_nth(weekday: Weekday, n: isize) -> Option<Self> {
        Self::create(weekday, Some(n))
    }

    fn is_valid_n(n: isize) -> bool {
        n != 0 && n < 500 && n > -500
    }
}

impl Display for WeekDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_prefix = match self.n {
            Some(n) => format!("{}", n),
            None => "".into(),
        };
        write!(f, "{}{}", n_prefix, self.weekday)
    }
}

#[derive(Error, Debug)]
pub enum InvalidWeekDayError {
    #[error("Invalid weekday specified: {0}")]
    InvalidWeekdayIdentifier(String),
    #[error("Malformed weekday: {0}")]
    Malformed(String),
}

impl FromStr for WeekDay {
    type Err = InvalidWeekDayError;

    fn from_str(day: &str) -> Result<Self, Self::Err> {
        use InvalidWeekDayError::Malformed;

        let e = Malformed(day.to_string());
        match day.len() {
            0..=2 => Err(e),
            3 => {
                let wday = Weekday::from_str(day).map_err(|_| Malformed(day.to_string()))?;
                WeekDay::new(wday).ok_or(e)
            }
            _ => {
                let wday = Weekday::from_str(&day[day.len() - 3..])
                    .map_err(|_| Malformed(day.to_string()))?;
                let n = day[0..day.len() - 3]
                    .parse::<isize>()
                    .map_err(|_| Malformed(day.to_string()))?;
                WeekDay::new_nth(wday, n).ok_or(e)
            }
        }
    }
}

impl Serialize for WeekDay {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WeekDay {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WeekDayVisitor;

        impl<'de> Visitor<'de> for WeekDayVisitor {
            type Value = WeekDay;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A valid string representation of weekday")
            }

            fn visit_str<E>(self, value: &str) -> Result<WeekDay, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<WeekDay>()
                    .map_err(|_| E::custom(format!("Malformed weekday: {}", value)))
            }
        }

        deserializer.deserialize_str(WeekDayVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_valid_weekday_str_correctly() {
        assert_eq!(
            "mon".parse::<WeekDay>().unwrap(),
            WeekDay::new(Weekday::Mon).unwrap()
        );
        assert_eq!(
            "sun".parse::<WeekDay>().unwrap(),
            WeekDay::new(Weekday::Sun).unwrap()
        );
        assert_eq!(
            "1mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, 1).unwrap()
        );
        assert_eq!(
            "17mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, 17).unwrap()
        );
        assert_eq!(
            "170mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, 170).unwrap()
        );
        assert_eq!(
            "+2mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, 2).unwrap()
        );
        assert_eq!(
            "+22mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, 22).unwrap()
        );
        assert_eq!(
            "-2mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, -2).unwrap()
        );
        assert_eq!(
            "-22mon".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(Weekday::Mon, -22).unwrap()
        );
    }

    #[test]
    fn parses_invalid_weekday_str_correctly() {
        assert!("".parse::<WeekDay>().is_err());
        assert!("-1".parse::<WeekDay>().is_err());
        assert!("7".parse::<WeekDay>().is_err());
        assert!("00".parse::<WeekDay>().is_err());
        assert!("-1!?".parse::<WeekDay>().is_err());
        assert!("-1WEDn".parse::<WeekDay>().is_err());
        assert!("-1mond".parse::<WeekDay>().is_err());
        assert!("mond".parse::<WeekDay>().is_err());
        assert!("1000mon".parse::<WeekDay>().is_err());
        assert!("0mon".parse::<WeekDay>().is_err());
        assert!("000mon".parse::<WeekDay>().is_err());
        assert!("+0mon".parse::<WeekDay>().is_err());
    }

    #[test]
    fn serializes_weekday() {
        assert_eq!(WeekDay::new(Weekday::Mon).unwrap().to_string(), "Mon");
        assert_eq!(WeekDay::new(Weekday::Tue).unwrap().to_string(), "Tue");
        assert_eq!(WeekDay::new(Weekday::Sun).unwrap().to_string(), "Sun");
        assert_eq!(
            WeekDay::new_nth(Weekday::Sun, 1).unwrap().to_string(),
            "1Sun"
        );
        assert_eq!(
            WeekDay::new_nth(Weekday::Sun, -1).unwrap().to_string(),
            "-1Sun"
        );
    }
}
