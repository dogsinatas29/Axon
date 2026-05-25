use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct IncludePathFix {
    pub file: String,
    pub original: String,
    pub fixed: String,
    pub reason: String,
}

pub fn normalize_includes(file_path: &str, content: &str) -> (String, Vec<IncludePathFix>) {
    let mut fixes = Vec::new();
    let mut result = content.to_string();
    
    let current_dir = Path::new(file_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("#include") {
            let (orig_include, target) = parse_include_line(trimmed);
            
            if let Some((target, _full_path)) = resolve_include_target(&target, &current_dir) {
                let relative_path = compute_relative_include(&current_dir, Path::new(&target));
                
                if relative_path != target {
                    let fixed_include = format!("#include \"{}\"", relative_path);
                    let old_line = line.to_string();
                    let new_line = line.replace(&orig_include, &fixed_include);
                    
                    result = result.replace(&old_line, &new_line);
                    
                    fixes.push(IncludePathFix {
                        file: file_path.to_string(),
                        original: orig_include.clone(),
                        fixed: fixed_include,
                        reason: format!("Normalized from '{}' to '{}' (relative path)", target, relative_path),
                    });
                }
            }
        }
    }
    
    (result, fixes)
}

fn parse_include_line(line: &str) -> (String, String) {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 3 {
        (format!("\"{}\"", parts[1]), parts[1].to_string())
    } else {
        let parts2: Vec<&str> = line.splitn(2, '<').collect();
        if parts2.len() >= 2 {
            let angle = parts2[1].trim_end_matches('>');
            (format!("<{}>", angle), angle.to_string())
        } else {
            (String::new(), String::new())
        }
    }
}

fn resolve_include_target(include: &str, current_dir: &Path) -> Option<(String, PathBuf)> {
    let search_paths = vec![
        current_dir.to_path_buf(),
        PathBuf::from("src"),
        PathBuf::from("include"),
        PathBuf::from("."),
    ];
    
    for base in search_paths {
        let candidate = base.join(include);
        if candidate.exists() {
            return Some((include.to_string(), candidate));
        }
        
        let candidate2 = PathBuf::from(include);
        if candidate2.exists() {
            return Some((include.to_string(), candidate2));
        }
    }
    
    None
}

fn compute_relative_include(_from_dir: &Path, target: &Path) -> String {
    let target_name = Path::new(target)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| target.to_string_lossy().to_string());
    
    target_name
}

pub fn build_include_policy() -> String {
    r#"
[C INCLUDE PATH POLICY]
When including headers in the same directory:
  ✅ CORRECT: #include "core.h"
  ❌ WRONG:  #include "src/core.h"
  ❌ WRONG:  #include "../src/core.h"

When including headers from subdirectory:
  ✅ CORRECT: #include "ui/menu.h"

When including system headers:
  ✅ CORRECT: #include <stdio.h>
  ❌ WRONG:  #include "stdio.h"

Rule: Use minimal relative path. Never include full project paths.
"#.trim().to_string()
}

pub fn parse_compile_error(error: &str) -> Option<(String, Vec<String>)> {
    let mut file = None;
    let mut messages = Vec::new();
    
    for line in error.lines() {
        if line.contains(".c:") || line.contains(".h:") {
            if let Some(pos) = line.find(".c:") {
                file = Some(line[..pos].to_string() + ".c");
            } else if let Some(pos) = line.find(".h:") {
                file = Some(line[..pos].to_string() + ".h");
            }
        }
        
        if line.contains("fatal error:") || line.contains("undefined reference") {
            messages.push(line.trim().to_string());
        }
    }
    
    file.map(|f| (f, messages))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_include() {
        // v0.0.31.02: Safely switch current directory to a temp folder to make resolution 100% deterministic
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = original_dir.join("debug").join("tmp_test_include");
        let _ = std::fs::create_dir_all(&temp_dir);
        std::env::set_current_dir(&temp_dir).unwrap();

        let _ = std::fs::create_dir_all("src/src");
        let _ = std::fs::write("src/core.h", "");
        let _ = std::fs::write("src/src/core.h", "");

        let content = r#"
#include "src/core.h"
void foo() {}
"#;
        let (fixed, _) = normalize_includes("src/main.c", content);

        // Restore dir first
        std::env::set_current_dir(original_dir).unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert!(fixed.contains("#include \"core.h\""));
    }
}