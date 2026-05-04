// === AXON GENERATED CODE ===
// Agent: junior-agent-2
// Task : 9f2993ae-2445-40d9-9ed7-15739451e710
// File : calculation.rs
// ===========================

pub fn calculate_age(year: i32, month: u8, day: u8) -> Option<i32> {
    let today = chrono::Local::today();
    let birth_date = chrono::Date::<chrono::Utc>::from_ymd_opt(year, month, day);
    
    if let Some(birth_date) = birth_date {
        let age = today.year() - birth_date.year();
        
        // Check for birthday in the current year
        if today.month() < month || (today.month() == month && today.day() < day) {
            age -= 1;
        }
        
        Some(age)
    } else {
        None
    }
}