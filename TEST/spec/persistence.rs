// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : 2be3806e-7567-4d37-8dd6-71ee16dcb80f
// File : persistence.rs
// ===========================

pub fn save_to_db(age: i32) {
    if age < 0 {
        panic!("Age cannot be negative");
    }

    // Simulate database insertion
    println!("Saving age {} to the database", age);
}
