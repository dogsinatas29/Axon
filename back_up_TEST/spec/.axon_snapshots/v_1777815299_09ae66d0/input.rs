pub fn get_day() -> i32 {
    chrono::Local::today().format("%-d").parse::<i32>().unwrap()
}

pub fn get_month() -> i32 {
    chrono::Local::today().format("%-m").parse::<i32>().unwrap()
}

pub fn get_year() -> i32 {
    chrono::Local::today().year()
}

pub fn get_name() -> String {
    "Lazy Architect-Gopher-fan".to_string()
}
