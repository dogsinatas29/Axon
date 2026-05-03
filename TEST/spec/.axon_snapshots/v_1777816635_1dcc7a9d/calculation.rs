// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : 8ce521d5-149f-45a5-b251-3199404c4201
// File : calculation.rs
// ===========================

pub fn calculate_age(year: i32, month: u8, day: u8) -> Option<i32> {
    let today = chrono::Local::today();
    let birth_date = chrono::Date::<chrono::Utc>::from_ymd_opt(year, month, day).unwrap_or_else(|| return None);
    
    if birth_date.year() > today.year() || (birth_date.year() == today.year() && birth_date.month() > today.month()) {
        return None;
    }

    Some(today.year() - birth_date.year() - if today.ordinal() < birth_date.ordinal() { 1 } else { 0 })
}
