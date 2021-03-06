use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use serde::{de::Visitor, Deserialize, Serialize};

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
    pub bysetpos: Option<Vec<isize>>,
    pub byweekday: Option<Vec<WeekDay>>,
    // by_month_day, by_month, by_year_day, by_week_no
    pub bymonthday: Option<Vec<isize>>,
    pub bymonth: Option<Vec<usize>>,
    pub byyearday: Option<Vec<isize>>,
    pub byweekno: Option<Vec<isize>>,
}

impl Default for RRuleOptions {
    fn default() -> Self {
        Self {
            freq: RRuleFrequenzy::Daily,
            interval: 1,
            byweekday: None,
            bysetpos: None,
            count: None,
            until: None,
            //
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
    weekday: usize,
}

impl WeekDay {
    fn create(weekday: usize, n: Option<isize>) -> Result<Self, ()> {
        if !Self::is_valid_weekday(weekday) {
            return Err(());
        }
        if let Some(n) = n {
            if !Self::is_valid_n(n) {
                return Err(());
            }
        }
        Ok(Self { weekday, n })
    }

    pub fn nth(&self) -> Option<isize> {
        self.n
    }
    pub fn weekday(&self) -> usize {
        self.weekday
    }

    pub fn new(weekday: usize) -> Result<Self, ()> {
        Self::create(weekday, None)
    }

    pub fn new_nth(weekday: usize, n: isize) -> Result<Self, ()> {
        Self::create(weekday, Some(n))
    }

    fn is_valid_n(n: isize) -> bool {
        n != 0 && n < 500 && n > -500
    }

    fn is_valid_weekday(wday: usize) -> bool {
        wday <= 6
    }
}

impl Display for WeekDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_prefix = match self.n {
            Some(n) => format!("{}", n),
            None => "".into(),
        };
        write!(f, "{}{}", n_prefix, weekday_to_str(self.weekday))
    }
}

fn str_to_weekday(d: &str) -> Result<usize, InvalidWeekDayError> {
    match d.to_uppercase().as_str() {
        "MO" => Ok(0),
        "TU" => Ok(1),
        "WE" => Ok(2),
        "TH" => Ok(3),
        "FR" => Ok(4),
        "SA" => Ok(5),
        "SU" => Ok(6),
        _ => Err(InvalidWeekDayError::InvalidWeekdayIdentifier(d.to_string())),
    }
}

fn weekday_to_str(wday: usize) -> String {
    match wday {
        0 => "MO",
        1 => "TU",
        2 => "WE",
        3 => "TH",
        4 => "FR",
        5 => "SA",
        6 => "SU",
        _ => "", // maybe use unreachable ?
    }
    .into()
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
        let e = InvalidWeekDayError::Malformed(day.to_string());
        if day.len() < 2 {
            return Err(e);
        } else if day.len() == 2 {
            // MO, TU, ...
            let wday = str_to_weekday(day)?;
            WeekDay::new(wday).map_err(|_| e)
        } else {
            let wday = str_to_weekday(&day[day.len() - 2..])?;
            let n = match day[0..day.len() - 2].parse::<isize>() {
                Ok(n) => n,
                Err(_) => return Err(e),
            };
            WeekDay::new_nth(wday, n).map_err(|_| e)
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
        assert_eq!("mo".parse::<WeekDay>().unwrap(), WeekDay::new(0).unwrap());
        assert_eq!("su".parse::<WeekDay>().unwrap(), WeekDay::new(6).unwrap());
        assert_eq!(
            "1mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, 1).unwrap()
        );
        assert_eq!(
            "17mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, 17).unwrap()
        );
        assert_eq!(
            "170mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, 170).unwrap()
        );
        assert_eq!(
            "+2mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, 2).unwrap()
        );
        assert_eq!(
            "+22mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, 22).unwrap()
        );
        assert_eq!(
            "-2mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, -2).unwrap()
        );
        assert_eq!(
            "-22mo".parse::<WeekDay>().unwrap(),
            WeekDay::new_nth(0, -22).unwrap()
        );
    }

    #[test]
    fn parses_invalid_weekday_str_correctly() {
        assert!("".parse::<WeekDay>().is_err());
        assert!("-1".parse::<WeekDay>().is_err());
        assert!("7".parse::<WeekDay>().is_err());
        assert!("00".parse::<WeekDay>().is_err());
        assert!("-1!?".parse::<WeekDay>().is_err());
        assert!("-1WED".parse::<WeekDay>().is_err());
        assert!("-1mon".parse::<WeekDay>().is_err());
        assert!("mon".parse::<WeekDay>().is_err());
        assert!("1000mo".parse::<WeekDay>().is_err());
        assert!("0mo".parse::<WeekDay>().is_err());
        assert!("000mo".parse::<WeekDay>().is_err());
        assert!("+0mo".parse::<WeekDay>().is_err());
    }

    #[test]
    fn serializes_weekday() {
        assert_eq!(WeekDay::new(0).unwrap().to_string(), "MO");
        assert_eq!(WeekDay::new(1).unwrap().to_string(), "TU");
        assert_eq!(WeekDay::new(6).unwrap().to_string(), "SU");
        assert_eq!(WeekDay::new_nth(6, 1).unwrap().to_string(), "1SU");
        assert_eq!(WeekDay::new_nth(6, -1).unwrap().to_string(), "-1SU");
    }
}
