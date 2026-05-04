// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : 290d71cc-8c29-4f08-86a5-b6d67e7b5319
// File : persistence.rs
// ===========================

pub fn save_to_db(age: i32) {
    if age < 0 {
        panic!("Age cannot be negative");
    }

    // Simulate database insertion
    println!("Saving age {} to the database", age);
}