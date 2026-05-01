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
}

impl Patch {
    pub fn new() -> Self {
        Self { files: vec![] }
    }
}
