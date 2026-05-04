// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : f304b2c1-aa88-43d0-b95d-fb4be72fbcd4
// File : input.rs
// ===========================

pub fn get_name() -> String {
    "John".to_string()
}

pub fn get_year() -> i32 {
    chrono::Local::now().year() as i32
}

pub fn get_day() -> u32 {
    chrono::Local::now().day() as u32
}

pub fn get_month() -> u32 {
    chrono::Local::now().month() as u32
}