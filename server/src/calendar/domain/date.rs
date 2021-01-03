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
    if year < 1970 || year > 2100 || month < 1 || month > 12 {
        return Err(anyhow::Error::msg(datestr));
    }

    let mut month_length = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if year % 400 == 0 || (year % 100 != 0 && year % 4 == 0) {
        month_length[1] = 29;
    }

    if day < 1 || day > month_length[month as usize - 1] {
        return Err(anyhow::Error::msg(datestr));
    }

    Ok((year, month, day))
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
            println!(
                "Checking date: {} with result {:?}",
                date,
                is_valid_date(date)
            );
            assert!(is_valid_date(date).is_err());
        }
    }
}
