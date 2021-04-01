use chrono::prelude::*;
use chrono_tz::Tz;

pub fn is_valid_date(datestr: &str) -> anyhow::Result<(i32, u32, u32)> {
    let datestr = String::from(datestr);
    let dates = datestr.split('-').collect::<Vec<_>>();
    if dates.len() != 3 {
        return Err(anyhow::Error::msg(datestr));
    }
    let year = dates[0].parse();
    let month = dates[1].parse();
    let day = dates[2].parse();

    if year.is_err() || month.is_err() || day.is_err() {
        return Err(anyhow::Error::msg(datestr));
    }

    let year = year.unwrap();
    let month = month.unwrap();
    let day = day.unwrap();
    if !(1970..=2100).contains(&year) || month < 1 || month > 12 {
        return Err(anyhow::Error::msg(datestr));
    }

    let month_length = get_month_length(year, month);

    if day < 1 || day > month_length {
        return Err(anyhow::Error::msg(datestr));
    }

    Ok((year, month, day))
}

pub fn is_leap_year(year: i32) -> bool {
    year % 400 == 0 || (year % 100 != 0 && year % 4 == 0)
}

// month: January -> 1
pub fn get_month_length(year: i32, month: u32) -> u32 {
    match month - 1 {
        0 => 31,
        1 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        2 => 31,
        3 => 30,
        4 => 31,
        5 => 30,
        6 => 31,
        7 => 31,
        8 => 30,
        9 => 31,
        10 => 30,
        11 => 31,
        _ => panic!("Invalid month"),
    }
}

pub fn format_date(date: &DateTime<Tz>) -> String {
    format!("{}-{}-{}", date.year(), date.month(), date.day())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_accepts_valid_dates() {
        let valid_dates = vec![
            "2018-1-1",
            "2025-12-31",
            "2020-1-12",
            "2020-2-29",
            "2020-02-2",
            "2020-02-02",
            "2020-2-09",
        ];

        for date in &valid_dates {
            assert!(is_valid_date(date).is_ok());
        }
    }

    #[test]
    fn it_rejects_invalid_dates() {
        let valid_dates = vec![
            "2018--1-1",
            "2020-1-32",
            "2020-2-30",
            "2020-0-1",
            "2020-1-0",
        ];

        for date in &valid_dates {
            assert!(is_valid_date(date).is_err());
        }
    }
}
