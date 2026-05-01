pub fn compute_age(year: i32, month: u8, day: u8) -> Option<i32> {
    let current_year = 2023;
    let current_month = 12;
    let current_day = 31;

    if year > current_year || (year == current_year && month >= current_month) {
        return None; // Invalid date
    }

    let age = current_year - year;
    if month < current_month || (month == current_month && day <= current_day) {
        Some(age)
    } else {
        Some(age - 1)
    }
}
