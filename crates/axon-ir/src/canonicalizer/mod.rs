pub fn canonicalize_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .trim_start_matches("/")
        .replace("\\", "/")
        .to_lowercase()
}

pub fn canonical_ir_name(path: &str) -> String {
    let trimmed = path.trim();
    let normalized = trimmed.replace("\\", "/");

    let filename = normalized.split('/').last().unwrap_or(&normalized);
    let name = filename
        .trim_end_matches(".c")
        .trim_end_matches(".h")
        .trim_end_matches(".cpp")
        .trim_end_matches(".hpp")
        .trim_end_matches(".rs")
        .trim_end_matches(".py")
        .trim_end_matches(".ts")
        .trim_end_matches(".js");

    name.to_lowercase()
}

pub fn normalize_paths(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    trimmed
        .split(|c| c == ',' || c == '|' || c == '/' || c == ' ')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

pub fn is_compound_path(path: &str) -> bool {
    let normalized = path.trim();
    let segments: Vec<&str> = normalized.split(|c| c == ',' || c == '|').collect();
    segments.len() > 1 && segments.iter().any(|s| s.contains(".c") || s.contains(".h"))
}

pub fn sanitize_llm_output(raw: &str) -> String {
    raw.chars()
        .filter(|c| {
            if c.is_control() {
                *c == '\n' || *c == '\r' || *c == '\t'
            } else {
                true
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_path() {
        assert_eq!(canonicalize_path("/main.c"), "main.c");
        assert_eq!(canonicalize_path("./main.c"), "main.c");
        assert_eq!(canonicalize_path("MAIN.C"), "main.c");
    }

    #[test]
    fn test_canonical_ir_name() {
        assert_eq!(canonical_ir_name("database.c"), "database");
        assert_eq!(canonical_ir_name("path/to/calculator.cpp"), "calculator");
    }
}