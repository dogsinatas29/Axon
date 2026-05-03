use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("axon.db")?;
    let mut stmt = conn.prepare("PRAGMA table_info(tasks)")?;
    let mut rows = stmt.query([])?;

    println!("--- Tasks Table Info ---");
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        println!("Column found: '{}'", name);
    }
    Ok(())
}
