use serde::{Serialize, Deserialize};
use super::error::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct FunctionSig {
    pub name: String,
    pub args: Vec<String>,
}

impl FunctionSig {
    pub fn from_signature_str(sig: &str) -> Option<Self> {
        let parts: Vec<&str> = sig.split('(').collect();
        if parts.len() != 2 { return None; }
        
        let name = parts[0].trim().to_string();
        let args_part = parts[1].trim_end_matches(')').trim();
        
        let args = if args_part.is_empty() {
            Vec::new()
        } else {
            args_part.split(',').map(|s| s.trim().to_string()).collect()
        };
        
        Some(Self { name, args })
    }
}

pub struct ValidationResult {
    pub ok: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self { ok: true, errors: Vec::new() }
    }
    
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self { ok: false, errors }
    }
}
