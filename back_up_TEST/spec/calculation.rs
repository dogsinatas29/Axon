pub fn calculate_age(year: u32, month: u32, day: u32) -> u32 {
    let today = chrono::Local::today();
    let birthday = chrono::NaiveDate::from_ymd(year as i32, month as i32, day as i32);
    let age_days = today.signed_julian_day() - birthday.signed_julian_day();
    return (age_days / 365) as u32;
}
