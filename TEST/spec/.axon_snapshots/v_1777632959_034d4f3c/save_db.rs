use std::collections::HashMap;

pub fn store_user(id: &str, age: i32) {
    let mut users = HashMap::new();
    if let Some(age) = users.insert(id, age) {
        println!("User with ID {} already exists and was updated.", id);
    } else {
        println!("User with ID {} has been added.", id);
    }
}

pub fn update_user(id: &str, age: i32) {
    let mut users = HashMap::new();
    if let Some(old_age) = users.insert(id, age) {
        println!("User with ID {} was updated from {} to {}", id, old_age, age);
    } else {
        println!("No user found with ID {}. No update performed.", id);
    }
}
