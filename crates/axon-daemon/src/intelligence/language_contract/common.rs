use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureIssue {
    PrimitiveAlias {
        original: String,
        replacement: String,
    },
    GenericType {
        type_name: String,
    },
    JavaCollection {
        type_name: String,
    },
    NamingConvention {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignatureFix {
    pub original: String,
    pub fixed: String,
    pub issues: Vec<SignatureIssue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixResult {
    Valid,
    AutoFixed(SignatureFix),
    SoftWarning(Vec<SignatureIssue>),
    HardFail(Vec<SignatureIssue>),
}

pub fn tokenize(sig: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut _in_generic = false;
    
    for ch in sig.chars() {
        match ch {
            '<' => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
                _in_generic = true;
                current.push(ch);
            }
            '>' => {
                current.push(ch);
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
                _in_generic = false;
            }
            ' ' | ',' | '(' | ')' | '*' | '&' | '[' | ']' => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
                if ch != ' ' && ch != ',' {
                    tokens.push(ch.to_string());
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    
    if !current.is_empty() {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            tokens.push(trimmed.to_string());
        }
    }
    
    tokens.retain(|t| !t.is_empty());
    tokens
}

pub fn safe_replace(sig: &str, from: &str, to: &str) -> String {
    let pattern = format!("\\b{}\\b", regex::escape(from));
    regex::Regex::new(&pattern)
        .map(|re| re.replace_all(sig, to).to_string())
        .unwrap_or_else(|_| sig.to_string())
}

pub fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    chars.first().map(|c| c.is_uppercase()).unwrap_or(false)
        && chars.iter().any(|c| c.is_lowercase())
        && !s.contains('_')
}

pub fn has_generic_syntax(s: &str) -> bool {
    s.contains('<') && s.contains('>')
}

pub fn is_java_collection(s: &str) -> bool {
    matches!(s, "List" | "ArrayList" | "HashMap" | "Map" | "Set" | "HashSet" | "LinkedList" | "Optional")
}

pub const C_PRIMITIVE_REPLACEMENTS: &[(&str, &str)] = &[
    ("String", "char*"),
    ("string", "char*"),
    ("boolean", "int"),
    ("Boolean", "int"),
    ("Integer", "int"),
    ("Double", "double"),
    ("Float", "float"),
    ("Object", "void*"),
    ("true", "1"),
    ("false", "0"),
];

pub const C_HARD_BANNED: &[&str] = &[
    "List<", "ArrayList<", "HashMap<", "Map<", "Set<", "HashSet<",
    "Optional<", "Stream<", "Queue<", "Deque<",
];

pub const C_SOFT_BANNED_PATTERNS: &[&str] = &[
    "getter", "setter", "toString", "hashCode",
];