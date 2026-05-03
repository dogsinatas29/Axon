// === AXON GENERATED CODE ===
// Agent: junior-agent-2
// Task : fe9c624f-0bd0-4eb9-98b0-c0b47215ad7f
// File : dependencies.rs
// ===========================

pub fn use_chrono() {
    let now = chrono::Local::now();
    println!("Current time: {}", now);
}

pub fn use_rusqlite() -> rusqlite::Result<()> {
    let conn = rusqlite::Connection::open_in_memory()?;
    conn.execute("CREATE TABLE test (id INTEGER)", [])?;
    conn.execute("INSERT INTO test VALUES (?)", [1])?;
    println!("Table created and data inserted.");

    Ok(())
}
