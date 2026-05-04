// === AXON GENERATED CODE ===
// Agent: junior-agent-2
// Task : 65c9fff7-1e94-47bd-acde-85e6247bfcb7
// File : validation.rs
// ===========================

pub fn validate_year(year: i32) -> bool {
    year >= 1 && year <= 9999
}

pub fn validate_day(day: i32) -> bool {
    day >= 1 && day <= 31
}

pub fn validate_month(month: i32) -> bool {
    month >= 1 && month <= 12
}