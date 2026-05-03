pub fn validate_month(month: u32) -> bool {
    month >= 1 && month <= 12
}

pub fn validate_day(day: u32) -> bool {
    day >= 1 && day <= 31
}

pub fn validate_year(year: u32) -> bool {
    year >= 1
}

