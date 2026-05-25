use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteDetectionResult {
    pub is_rewrite: bool,
    pub score: f32,
    pub violations: Vec<RewriteViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RewriteViolation {
    EntireFileReplaced,
    FunctionDeleted,
    ImportWipe,
    NamespaceWipe,
    InterfaceCollapse,
    ComponentDeletion,
    HighMutationRatio { ratio: f32, threshold: f32 },
}

pub struct RewriteDetector;

impl RewriteDetector {
    pub fn detect(old_content: &str, new_content: &str) -> RewriteDetectionResult {
        let mut violations = Vec::new();
        let mut score = 0.0f32;

        if Self::is_entire_file_replaced(old_content, new_content) {
            violations.push(RewriteViolation::EntireFileReplaced);
            score += 0.8;
        }

        if Self::has_function_deletion(old_content, new_content) {
            violations.push(RewriteViolation::FunctionDeleted);
            score += 0.5;
        }

        if Self::has_import_wipe(old_content, new_content) {
            violations.push(RewriteViolation::ImportWipe);
            score += 0.4;
        }

        if Self::has_namespace_wipe(old_content, new_content) {
            violations.push(RewriteViolation::NamespaceWipe);
            score += 0.4;
        }

        if Self::has_interface_collapse(old_content, new_content) {
            violations.push(RewriteViolation::InterfaceCollapse);
            score += 0.5;
        }

        if Self::has_component_deletion(old_content, new_content) {
            violations.push(RewriteViolation::ComponentDeletion);
            score += 0.6;
        }

        let mutation_ratio = Self::calculate_mutation_ratio(old_content, new_content);
        if mutation_ratio > 0.7 {
            violations.push(RewriteViolation::HighMutationRatio {
                ratio: mutation_ratio,
                threshold: 0.7,
            });
            score += mutation_ratio * 0.5;
        }

        score = score.min(1.0);

        let is_rewrite = !violations.is_empty();

        RewriteDetectionResult {
            is_rewrite,
            score,
            violations,
        }
    }

    fn is_entire_file_replaced(old_content: &str, new_content: &str) -> bool {
        let old_lines = old_content.lines().count();
        let new_lines = new_content.lines().count();

        if old_lines < 10 || new_lines < 10 {
            return false;
        }

        let old_lower = old_content.to_lowercase();
        let new_lower = new_content.to_lowercase();

        let common_tokens = ["fn ", "pub fn", "class ", "struct ", "impl ", "def ", "int ", "void "];
        let old_has_code = common_tokens.iter().any(|t| old_lower.contains(*t));
        let new_has_code = common_tokens.iter().any(|t| new_lower.contains(*t));

        if !old_has_code || !new_has_code {
            return false;
        }

        let old_funcs = Self::extract_function_signatures(old_content);
        let new_funcs = Self::extract_function_signatures(new_content);

        if old_funcs.len() >= 3 && new_funcs.len() >= 3 {
            let retained: usize = old_funcs.iter().filter(|f| new_funcs.contains(f)).count();
            let retention_ratio = retained as f32 / old_funcs.len() as f32;

            if retention_ratio < 0.2 {
                return true;
            }
        }

        false
    }

    fn has_function_deletion(old_content: &str, new_content: &str) -> bool {
        let old_funcs = Self::extract_function_signatures(old_content);
        let new_funcs = Self::extract_function_signatures(new_content);

        if old_funcs.is_empty() {
            return false;
        }

        let deleted: usize = old_funcs.iter().filter(|f| !new_funcs.contains(f)).count();
        let deletion_ratio = deleted as f32 / old_funcs.len() as f32;

        deletion_ratio > 0.5
    }

    fn has_import_wipe(old_content: &str, new_content: &str) -> bool {
        let old_imports = Self::extract_imports(old_content);
        let new_imports = Self::extract_imports(new_content);

        if old_imports.is_empty() {
            return false;
        }

        let wipe_ratio = if new_imports.is_empty() {
            1.0
        } else {
            let retained: usize = old_imports.iter().filter(|i| new_imports.contains(i)).count();
            let wipe = (old_imports.len() - retained) as f32 / old_imports.len() as f32;
            wipe
        };

        wipe_ratio > 0.7
    }

    fn has_namespace_wipe(old_content: &str, new_content: &str) -> bool {
        let old_ns = Self::extract_namespaces(old_content);
        let new_ns = Self::extract_namespaces(new_content);

        if old_ns.is_empty() {
            return false;
        }

        if new_ns.is_empty() && old_ns.len() >= 2 {
            return true;
        }

        let retained: usize = old_ns.iter().filter(|n| new_ns.contains(n)).count();
        let wipe_ratio = (old_ns.len() - retained) as f32 / old_ns.len() as f32;

        wipe_ratio > 0.8
    }

    fn has_interface_collapse(old_content: &str, new_content: &str) -> bool {
        let old_structs = Self::extract_structs(old_content);
        let new_structs = Self::extract_structs(new_content);

        if old_structs.is_empty() {
            return false;
        }

        let collapse_ratio = if new_structs.is_empty() {
            1.0
        } else {
            let retained: usize = old_structs.iter().filter(|s| new_structs.contains(s)).count();
            (old_structs.len() - retained) as f32 / old_structs.len() as f32
        };

        collapse_ratio > 0.7
    }

    fn has_component_deletion(old_content: &str, new_content: &str) -> bool {
        let old_components = Self::extract_components(old_content);
        let new_components = Self::extract_components(new_content);

        if old_components < 3 {
            return false;
        }

        if new_components == 0 {
            return old_components >= 2;
        }

        let deletion_ratio = (old_components - new_components) as f32 / old_components as f32;
        deletion_ratio > 0.6
    }

    fn calculate_mutation_ratio(old_content: &str, new_content: &str) -> f32 {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        if old_lines.is_empty() {
            return if new_lines.is_empty() { 0.0 } else { 1.0 };
        }

        if new_lines.is_empty() {
            return 1.0;
        }

        let max_len = old_lines.len().max(new_lines.len());

        if max_len < 5 {
            return 0.0;
        }

        let mut changed = 0;
        let compare_len = old_lines.len().min(new_lines.len());

        for i in 0..compare_len {
            if old_lines[i] != new_lines[i] {
                changed += 1;
            }
        }

        let extra_lines = (new_lines.len() - compare_len).max(old_lines.len() - compare_len);
        changed += extra_lines;

        changed as f32 / max_len as f32
    }

    fn extract_function_signatures(content: &str) -> Vec<String> {
        let mut funcs = Vec::new();

        let patterns = [
            r"(?m)^(pub )?fn \w+",
            r"(?m)^(pub )?async fn \w+",
            r"(?m)^\w+ \w+ \([^)]*\) \{",
            r"(?m)^void \w+ \(",
            r"(?m)^( *)def \w+",
            r"(?m)^( *)async def \w+",
            r"(?m)^\w+ \w+::\w+",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(content) {
                    let sig = cap.as_str().trim().to_string();
                    if !sig.is_empty() && !funcs.contains(&sig) {
                        funcs.push(sig);
                    }
                }
            }
        }

        funcs
    }

    fn extract_imports(content: &str) -> Vec<String> {
        let mut imports = Vec::new();

        let re_include_angle = regex::Regex::new(r"(?m)^#include <[^>]+>").unwrap();
        let re_include_quote = regex::Regex::new(r#"(?m)^#include "[^"]+""#).unwrap();
        let re_use = regex::Regex::new(r"(?m)^use \w+").unwrap();
        let re_import = regex::Regex::new(r"(?m)^import \w+").unwrap();
        let re_from_import = regex::Regex::new(r"(?m)^from \w+ import").unwrap();

        for cap in re_include_angle.find_iter(content) {
            let line = cap.as_str().trim().to_string();
            if !line.is_empty() && !imports.contains(&line) {
                imports.push(line);
            }
        }
        for cap in re_include_quote.find_iter(content) {
            let line = cap.as_str().trim().to_string();
            if !line.is_empty() && !imports.contains(&line) {
                imports.push(line);
            }
        }
        for cap in re_use.find_iter(content) {
            let line = cap.as_str().trim().to_string();
            if !line.is_empty() && !imports.contains(&line) {
                imports.push(line);
            }
        }
        for cap in re_import.find_iter(content) {
            let line = cap.as_str().trim().to_string();
            if !line.is_empty() && !imports.contains(&line) {
                imports.push(line);
            }
        }
        for cap in re_from_import.find_iter(content) {
            let line = cap.as_str().trim().to_string();
            if !line.is_empty() && !imports.contains(&line) {
                imports.push(line);
            }
        }

        imports
    }

    fn extract_namespaces(content: &str) -> Vec<String> {
        let mut namespaces = Vec::new();

        let patterns = [
            r"(?m)^namespace \w+",
            r"(?m)^module \w+",
            r"(?m)^package [\w.]+",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(content) {
                    if let Some(name) = cap.as_str().split_whitespace().nth(1) {
                        let ns = name.trim().to_string();
                        if !ns.is_empty() && !namespaces.contains(&ns) {
                            namespaces.push(ns);
                        }
                    }
                }
            }
        }

        namespaces
    }

    fn extract_structs(content: &str) -> Vec<String> {
        let mut structs = Vec::new();

        let patterns = [
            r"(?m)^struct \w+",
            r"(?m)^class \w+",
            r"(?m)^enum \w+",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(content) {
                    if let Some(name) = cap.as_str().split_whitespace().nth(1) {
                        let s = name.trim().to_string();
                        if !s.is_empty() && !structs.contains(&s) {
                            structs.push(s);
                        }
                    }
                }
            }
        }

        structs
    }

    fn extract_components(content: &str) -> usize {
        let structs = Self::extract_structs(content);
        let funcs = Self::extract_function_signatures(content);
        let imports = Self::extract_imports(content);

        structs.len() + funcs.len() / 2 + imports.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entire_file_replaced() {
        let old = "pub fn init() { }\npub fn process() { }\npub fn cleanup() { }\npub fn render() { }\npub fn update() { }\npub fn draw() { }\npub fn handle() { }\npub fn execute() { }\npub fn dispatch() { }\npub fn process_event() { }";
        let new = "pub fn start() { }\npub fn execute() { }\npub fn terminate() { }\npub fn draw_now() { }\npub fn refresh() { }\npub fn display() { }\npub fn manage() { }\npub fn run() { }\npub fn send() { }\npub fn handle_event() { }";

        let result = RewriteDetector::detect(old, new);
        assert!(result.is_rewrite);
        assert!(result.violations.contains(&RewriteViolation::EntireFileReplaced));
    }

    #[test]
    fn test_function_deletion() {
        let old = "fn get_year() -> i32 { 2023 }\nfn get_name() -> String { \"Original\".to_string() }\nfn get_month() -> u8 { 1 }\nfn get_day() -> u8 { 1 }";
        let new = "fn get_year() -> i32 { 2025 }";

        let result = RewriteDetector::detect(old, new);
        assert!(result.is_rewrite);
        assert!(result.violations.contains(&RewriteViolation::FunctionDeleted));
    }

    #[test]
    fn test_normal_modification() {
        let old = "fn get_year() -> i32 { 2023 }\nfn get_name() -> String { \"Original\".to_string() }";
        let new = "fn get_year() -> i32 { 2025 }\nfn get_name() -> String { \"Updated\".to_string() }";

        let result = RewriteDetector::detect(old, new);
        assert!(!result.is_rewrite);
    }
}