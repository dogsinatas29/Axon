use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub symbol: String,
    pub parameter_count: usize,
    pub has_return_type: bool,
    pub is_public: bool,
}

impl SignatureInfo {
    pub fn extract_naive(code_snippet: &str, symbol: &str) -> Option<Self> {
        let is_public = code_snippet.contains("pub fn") || code_snippet.contains("pub struct");
        
        let param_start = code_snippet.find('(')?;
        let param_end = code_snippet.find(')')?;
        
        let params_str = &code_snippet[param_start + 1..param_end];
        let parameter_count = if params_str.trim().is_empty() {
            0
        } else {
            params_str.split(',').count()
        };

        let has_return_type = code_snippet[param_end..].contains("->");

        Some(Self {
            symbol: symbol.to_string(),
            parameter_count,
            has_return_type,
            is_public,
        })
    }
}

pub struct SignatureDriftValidator;

impl SignatureDriftValidator {
    pub fn check_drift(original: &SignatureInfo, patched: &SignatureInfo) -> Result<(), String> {
        let mut violations = Vec::new();
        
        if original.parameter_count != patched.parameter_count {
            violations.push(format!("Parameter count changed: {} -> {}", original.parameter_count, patched.parameter_count));
        }
        
        if original.has_return_type != patched.has_return_type {
            violations.push(format!("Return type existence changed: {} -> {}", original.has_return_type, patched.has_return_type));
        }
        
        if original.is_public != patched.is_public {
            violations.push(format!("Visibility changed: pub {} -> pub {}", original.is_public, patched.is_public));
        }

        if !violations.is_empty() {
            return Err(format!("Signature Drift Detected: {:?}", violations));
        }

        Ok(())
    }
}
