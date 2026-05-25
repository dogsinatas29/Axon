use std::path::Path;
use std::io::Write;
use std::fs::{File, OpenOptions};

/// Writes a JSON file atomically, ensuring crash consistency.
pub fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    
    // Generate a unique temp file path
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    let temp_path = parent.join(format!(".{}.{}.tmp", filename, timestamp));
    
    // 1. Write to temp file
    let mut temp_file = File::create(&temp_path)?;
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    temp_file.write_all(json.as_bytes())?;
    
    // 2. Fsync temp file to ensure data is physically on disk
    temp_file.sync_all()?;
    
    // 3. Atomic rename (overwrites the old file cleanly)
    std::fs::rename(&temp_path, path)?;
    
    // 4. Fsync parent directory to ensure the directory entry rename is flushed
    let dir = File::open(parent)?;
    dir.sync_all()?;
    
    Ok(())
}

/// Appends a JSONL record atomically (forces fsync after append).
pub fn append_jsonl_atomic<T: serde::Serialize>(path: &Path, record: &T) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let json = serde_json::to_string(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writeln!(file, "{}", json)?;
    
    // Fsync immediately after append
    file.sync_all()?;
    
    Ok(())
}
