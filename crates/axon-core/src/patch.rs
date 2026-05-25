use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PatchAction {
    Rewrite,
    Append,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePatch {
    pub path: String,
    pub action: PatchAction,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Patch {
    pub files: Vec<FilePatch>,
    pub thought: Option<String>,
}

impl Patch {
    pub fn new() -> Self {
        Self { files: vec![], thought: None }
    }
}

// Phase 8: Transaction Envelope — patch integrity metadata
// Ensures: BEGIN/END markers present, BYTE_COUNT matches, CHECKSUM valid
#[derive(Debug, Clone)]
pub struct PatchEnvelope {
    pub patch_id: String,
    pub target: String,
    pub patch_version: u32,
    pub hunk_count: u32,
    pub byte_count: usize,
    pub checksum: String, // simple CRC-like hash of body content
    pub body: String,
    pub is_complete: bool,
    pub integrity_errors: Vec<String>,
}

impl PatchEnvelope {
    pub fn new() -> Self {
        Self {
            patch_id: String::new(),
            target: String::new(),
            patch_version: 2,
            hunk_count: 0,
            byte_count: 0,
            checksum: String::new(),
            body: String::new(),
            is_complete: false,
            integrity_errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.integrity_errors.is_empty()
    }

    pub fn compute_checksum(body: &str) -> String {
        // Simple djb2 hash for integrity check
        let mut hash: u64 = 5381;
        for b in body.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        format!("{:016x}", hash)
    }

    pub fn validate(&mut self) {
        // Check BEGIN marker was present (set by parser)
        if self.patch_id.is_empty() {
            self.integrity_errors.push("missing_PATCH_ID".to_string());
        }
        if self.target.is_empty() {
            self.integrity_errors.push("missing_TARGET".to_string());
        }
        // Check END marker was present
        if !self.is_complete {
            self.integrity_errors.push("missing_PATCH_END".to_string());
        }
        // Check body is not empty
        if self.body.trim().is_empty() {
            self.integrity_errors.push("empty_PATCH_BODY".to_string());
        }
        // Optional: Validate BYTE_COUNT (only if declared > 0)
        if self.byte_count > 0 {
            let actual_bytes = self.body.len();
            if self.byte_count != actual_bytes {
                self.integrity_errors.push(format!("BYTE_COUNT_mismatch declared={} actual={}", self.byte_count, actual_bytes));
            }
        }
        // Optional: Validate CHECKSUM (only if declared and looks like a real hash)
        if !self.checksum.is_empty() && self.checksum.len() == 16 {
            let expected = Self::compute_checksum(&self.body);
            if self.checksum != expected {
                self.integrity_errors.push(format!("CHECKSUM_mismatch declared={} actual={}", self.checksum, expected));
            }
        }
    }
}
