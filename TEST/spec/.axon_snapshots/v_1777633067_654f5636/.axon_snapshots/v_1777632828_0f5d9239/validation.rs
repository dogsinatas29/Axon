use std::result;

pub fn validate_year(year: i32) -> Result<(), String> {
    if year < 1900 || year > 2100 {
        Err("Invalid year".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_month(month: u8) -> Result<(), String> {
    if month == 0 || month > 12 {
        Err("Invalid month".to_string())
    } else {
        Ok(())
    }
}

pub fn validate_day(day: u8, month: u8, year: i32) -> Result<(), String> {
    use chrono::NaiveDate;

    let date = NaiveDate::from_ymd_opt(year, month, day);
    match date {
        Some(_) => Ok(()),
        None => Err("Invalid day".to_string()),
    }
}
