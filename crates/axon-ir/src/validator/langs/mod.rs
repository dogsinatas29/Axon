pub mod c;
pub mod python;
pub mod rust;
pub mod lua;

use crate::schema::ProjectIR;
use crate::validator::ValidationError;

pub trait LanguageValidator: Send + Sync {
    fn validate(&self, ir: &ProjectIR) -> Vec<ValidationError>;
    fn check_entry_point(&self, files: &[(String, String)]) -> bool;
}

pub fn get_validator(language: &str) -> Option<Box<dyn LanguageValidator>> {
    match language.to_lowercase().as_str() {
        "c" | "cpp" => Some(Box::new(c::CValidator)),
        "python" => Some(Box::new(python::PythonValidator)),
        "rust" => Some(Box::new(rust::RustValidator)),
        "lua" => Some(Box::new(lua::LuaValidator)),
        _ => None,
    }
}
