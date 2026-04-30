use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IR {
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub responsibility: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub returns: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub path: String,
    pub kind: String,
}

pub fn validate(ir: &IR) -> Vec<ValidationError> {
    let mut errors = vec![];

    for (i, file) in ir.files.iter().enumerate() {
        if file.name.trim().is_empty() {
            errors.push(err(format!("files[{}].name", i), "EmptyField"));
        }

        for (j, func) in file.functions.iter().enumerate() {
            if func.name.trim().is_empty() {
                errors.push(err(format!("files[{}].functions[{}].name", i, j), "EmptyField"));
            }
        }
    }

    errors
}

fn err(path: String, kind: &str) -> ValidationError {
    ValidationError {
        path,
        kind: kind.to_string(),
    }
}
