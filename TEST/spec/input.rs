// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : 1dcc7a9d-d4fd-452f-a530-9d1c8368f23e
// File : input.rs
// ===========================

pub fn get_name() -> String {
    "김철수".to_string()
}

pub fn get_day() -> u32 {
    let today = chrono::Local::now();
    today.day()
}

pub fn get_month() -> u32 {
    let today = chrono::Local::now();
    today.month()
}

pub fn get_year() -> i32 {
    let today = chrono::Local::now();
    today.year()
}
