use std::collections::HashMap;

pub fn query_db(name: &str) -> Result<bool, String> {
    let mut db: HashMap<String, bool> = HashMap::new();
    
    // Example data for demonstration purposes
    db.insert("Alice".to_string(), true);
    db.insert("Bob".to_string(), false);

    match db.get(name) {
        Some(&exists) => Ok(exists),
        None => Err(format!("User {} not found", name)),
    }
}
