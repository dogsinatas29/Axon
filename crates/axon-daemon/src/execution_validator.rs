use std::process::Command;
use std::path::Path;
use axon_ir::ProjectIR;

pub enum ValidationMode {
    Incremental,
    Full,
}

pub fn validate(project_root: &str, file_path: &str, mode: ValidationMode, ir: Option<&ProjectIR>) -> anyhow::Result<()> {
    // v0.0.29: Structural Validation (Semantic completeness & safety)
    analyze_structural_integrity(project_root, file_path, ir)?;

    if file_path.ends_with(".rs") {
        validate_rust(project_root, mode)
    } else if file_path.ends_with(".py") {
        validate_python(project_root, file_path)
    } else if file_path.ends_with(".c") || file_path.ends_with(".h") {
        validate_c(project_root, file_path, ir)
    } else {
        Ok(())
    }
}

/// v0.0.29: Semantic & Structural Integrity Analysis
/// Detects stubs, header body violations, and hallucinated patterns before compilation.
fn analyze_structural_integrity(project_root: &str, file_path: &str, ir: Option<&ProjectIR>) -> anyhow::Result<()> {
    let path = Path::new(project_root).join(file_path);
    if !path.exists() { return Ok(()); }
    
    let content = std::fs::read_to_string(&path)?;
    
    // 1. Stub & TODO Detection (Anti-Stub v3)
    let stub_patterns = [
        "// todo", "// implement", "// ...", 
        "/* todo", "/* implement", "/* ...",
        "// add logic", "/* add logic",
        "// TODO", "FIXME", "panic!", "unimplemented!"
    ];
    
    for pattern in &stub_patterns {
        if content.contains(pattern) {
            return Err(anyhow::anyhow!(
                "SEMANTIC VIOLATION: Stub detected in '{}' (Pattern: '{}'). Implementation is incomplete.",
                file_path, pattern
            ));
        }
    }

    // 2. Language-Specific Structural Rules
    if file_path.ends_with(".h") {
        // C Header Rules: Forbid function bodies, require guards
        if content.contains('{') && (content.contains("if") || content.contains("while") || content.contains("return")) {
            return Err(anyhow::anyhow!(
                "STRUCTURAL VIOLATION: Function body detected in header file '{}'. Headers must only contain declarations.",
                file_path
            ));
        }
        if !content.contains("#ifndef") && !content.contains("#define") && !content.contains("#pragma once") {
             return Err(anyhow::anyhow!(
                "STRUCTURAL VIOLATION: Missing header guards in '{}'. Enforce #ifndef/#define pattern.",
                file_path
            ));
        }
    }

    if file_path.ends_with("main.c") {
        // Entry Point Rules: Detect "Hello World" fallback
        if !content.contains("while") && !content.contains("for") && !content.contains("init") {
            if content.len() < 150 {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: 'main.c' appears to be a placeholder/fallback. Entry point must implement actual system orchestration logic (loops, init calls).",
                ));
            }
        }
    }

    // 3. Architecture Alignment (IR Contract Enforcement)
    if let Some(ir_data) = ir {
        // A. Interface Drift Check
        // A. Interface Drift Check (v0.0.29: ABI Signature Matching)
        if let Some(comp) = ir_data.components.get(file_path) {
            for (_, func) in &comp.functions {
                // v0.0.29: Ensure the EXACT function signature is present in the file
                if !content.contains(&func.signature) {
                    tracing::warn!("⚠️ [ABI_DRIFT] Missing signature in {}: expected '{}'", file_path, func.signature);
                    return Err(anyhow::anyhow!(
                        "CONTRACT DRIFT: ABI Signature mismatch in '{}'.\nExpected: '{}'\nImplementation might have hallucinated arguments or return types.",
                        file_path, func.signature
                    ));
                }
            }
            for line in content.lines() {
            if line.trim().starts_with("#include \"") {
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 2 {
                    let h_file = parts[1];
                    // Skip standard libs if quoted (rare but possible)
                    if h_file.ends_with(".h") {
                        // Check if this header exists in the IR or physically exists in include/
                        let h_path_ir = format!("include/{}", h_file);
                        let h_path_phys = Path::new(project_root).join("include").join(h_file);
                        let h_path_rel = Path::new(project_root).join(h_file);

                        let mut found = false;
                        if ir_data.components.contains_key(&h_path_ir) || ir_data.components.contains_key(h_file) { found = true; }
                        if h_path_phys.exists() || h_path_rel.exists() { found = true; }
                        
                        if !found {
                            return Err(anyhow::anyhow!(
                                "HALLUCINATION DETECTED: Illegal include '#include \"{}\"' found in '{}'. This file is not in the architecture IR.",
                                h_file, file_path
                            ));
                        }
                    }
                }
            }
        } // End of include protection loop
        
        // C. Dependency Discipline Check (v0.0.28)
        for inc in &comp.forbidden_includes {
            if content.contains(&format!("#include \"{}\"", inc)) || content.contains(&format!("#include <{}>", inc)) {
                return Err(anyhow::anyhow!(
                    "CONTRACT VIOLATION: Forbidden include '{}' found in file '{}'. This violates the architectural isolation rules.",
                    inc, file_path
                ));
            }
        }

        // D. Logic Isolation Check (v0.0.28)
        for sym in &comp.forbidden_symbols {
            if content.contains(sym) {
                return Err(anyhow::anyhow!(
                    "CONTRACT VIOLATION: Forbidden symbol/logic '{}' found in file '{}'. This module MUST NOT contain this logic.",
                    sym, file_path
                ));
            }
        }
        // E. Entrypoint Integrity Gate (v0.0.28)
        if comp.is_entrypoint && ir_data.components.len() > 1 {
            let mut calls_others = false;
            for (other_path, other_comp) in &ir_data.components {
                if other_path == file_path { continue; }
                for func in other_comp.functions.values() {
                    if content.contains(&func.name) {
                        calls_others = true;
                        break;
                    }
                }
                if calls_others { break; }
            }

            if !calls_others {
                return Err(anyhow::anyhow!(
                    "ENTRYPOINT COLLAPSE: 'main.c' is a trivial placeholder and does NOT integrate with other modules. Global integration logic is missing.",
                ));
            }
        }
    } // End if let Some(comp)
    } // End if let Some(ir_data)

    Ok(())
}

fn validate_c(project_root: &str, file_path: &str, _ir: Option<&ProjectIR>) -> anyhow::Result<()> {
    let path = Path::new(project_root).join(file_path);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    
    // STRICT CONTRACT: Require .h file for every .c module (except main.c and main_*.c)
    let is_main_file = file_stem == "main" || file_stem.starts_with("main_");
    if !is_main_file {
        // v0.0.28: Architecture-Aware Header Resolution
        // 1. Check same directory as .c file
        let h_path_same = path.with_extension("h");
        // 2. Check root 'include/' directory
        let h_path_include = Path::new(project_root).join("include").join(format!("{}.h", file_stem));
        
        if !h_path_same.exists() && !h_path_include.exists() {
            return Err(anyhow::anyhow!(
                "STRICT CONTRACT VIOLATION: Corresponding header file for '{}' is missing. Interface Separation Principle requires all .c files to have a .h file. (Checked: '{}.h' and 'include/{}.h')",
                file_path, file_stem, file_stem
            ));
        }
    }

    // v0.0.29: Sequential Integrity Check
    // ONLY trigger full CMake build if:
    // 1. CMakeLists.txt exists
    // 2. ALL components defined in IR actually exist as files (Phase 3 completion check)
    let mut all_files_exist = true;
    if let Some(ir_data) = _ir {
        for (f_path, _) in &ir_data.components {
             if !Path::new(project_root).join(f_path).exists() {
                 all_files_exist = false;
                 break;
             }
        }
    } else {
        all_files_exist = false; // No IR context, cannot verify completeness
    }

    if Path::new(project_root).join("CMakeLists.txt").exists() && all_files_exist {
        validate_c_project(project_root)?;
    } else {
        // v0.0.29: Fallback to individual file syntax check if project is incomplete
        tracing::info!("🧪 [SEQUENTIAL_VALIDATION] Project incomplete. Performing individual syntax check for '{}'", file_path);
        let output = Command::new("gcc")
            .arg("-fsyntax-only")
            .arg("-Iinclude") // v0.0.29: Include header path for individual check
            .arg(file_path)
            .current_dir(project_root)
            .output()?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            // v0.0.29: Materialize terminal log for Boss Board
            let _ = std::fs::create_dir_all(Path::new(project_root).join("debug"));
            let _ = std::fs::write(Path::new(project_root).join("debug/last_compiler_error.txt"), &*err);
            return Err(anyhow::anyhow!("C syntax check failed for {}:\n{}", file_path, err));
        }
    }

    Ok(())
}

fn validate_c_project(project_root: &str) -> anyhow::Result<()> {
    let build_dir = Path::new(project_root).join("build");
    let _ = std::fs::create_dir_all(&build_dir);
    
    // 1. CMake Configure
    let cmake_output = Command::new("cmake")
        .arg("..")
        .current_dir(&build_dir)
        .output()?;
        
    if !cmake_output.status.success() {
        let err = String::from_utf8_lossy(&cmake_output.stderr);
        return Err(anyhow::anyhow!("CMake configuration failed:\n{}", err));
    }
    
    // 2. Build (Make)
    let make_output = Command::new("make")
        .current_dir(&build_dir)
        .output()?;
        
    if !make_output.status.success() {
        let err = String::from_utf8_lossy(&make_output.stderr);
        // v0.0.29: Materialize terminal log for Boss Board
        let _ = std::fs::create_dir_all(Path::new(project_root).join("debug"));
        let _ = std::fs::write(Path::new(project_root).join("debug/last_compiler_error.txt"), &*err);
        // Stage 4: Critical feedback for the agent
        return Err(anyhow::anyhow!("Build failed. COMPILER ERROR:\n{}", err));
    }
    
    Ok(())
}

fn validate_rust(project_root: &str, mode: ValidationMode) -> anyhow::Result<()> {
    // v0.0.28 Safety: ONLY run cargo check if a local Cargo.toml exists.
    if !Path::new(project_root).join("Cargo.toml").exists() {
        tracing::warn!("🍃 [SKIP_CHECK] No local Cargo.toml found in {}. Skipping 'cargo check'.", project_root);
        return Ok(());
    }

    // Phase 1: Try incremental (Native cargo check)
    if let ValidationMode::Full = mode {
        let _ = Command::new("cargo")
            .arg("clean")
            .current_dir(project_root)
            .status();
    }

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(project_root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        // Phase 2: If incremental fails, it might be a real error or cache issue (rare)
        // In full mode, we've already cleaned, so this is a real error.
        match mode {
            ValidationMode::Incremental => {
                tracing::warn!("⚠️ [INCREMENTAL_FAIL] Cargo check failed. Retrying with FULL CLEAN to verify...");
                validate_rust(project_root, ValidationMode::Full)
            }
            ValidationMode::Full => {
                Err(anyhow::anyhow!("Rust compilation check failed after full clean."))
            }
        }
    }
}

fn validate_python(project_root: &str, file_path: &str) -> anyhow::Result<()> {
    // Python incremental: Check specific file syntax and main entry point
    let status = Command::new("python3")
        .arg("-m")
        .arg("py_compile")
        .arg(file_path)
        .current_dir(project_root)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Python syntax check failed for {}", file_path));
    }

    // Also check entry point integrity if it exists
    if Path::new(project_root).join("main.py").exists() {
        let entry_status = Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg("main.py")
            .current_dir(project_root)
            .status()?;
            
        if !entry_status.success() {
            return Err(anyhow::anyhow!("Python entry point 'main.py' is broken."));
        }
    }

    Ok(())
}

pub fn selective_run(project_root: &str, file_path: &str, targets: Vec<String>) -> anyhow::Result<()> {
    if targets.is_empty() {
        tracing::info!("🍃 [SELECTIVE_RUN] Skipping runtime validation for Pure nodes.");
        return Ok(());
    }

    tracing::info!("🧪 [SELECTIVE_RUN] Running runtime validation for: {:?}", targets);
    
    if file_path.ends_with(".rs") {
        // Rust: Typically cargo test or run
        // v0.0.28 Safety: ONLY run cargo test if a local Cargo.toml exists to prevent sandbox escape.
        if Path::new(project_root).join("Cargo.toml").exists() {
            let status = Command::new("cargo")
                .arg("test")
                .current_dir(project_root)
                .status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Rust runtime validation (tests) failed."));
            }
        } else {
            tracing::warn!("🍃 [SKIP_TEST] No local Cargo.toml found in {}. Skipping 'cargo test' to prevent sandbox escape.", project_root);
        }
    } else if file_path.ends_with(".py") {
        // Python: Run main.py as a smoke test
        if Path::new(project_root).join("main.py").exists() {
            let status = Command::new("python3")
                .arg("main.py")
                .current_dir(project_root)
                .status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Python runtime validation (main.py) failed."));
            }
        }
    }

    Ok(())
}

pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// v0.0.29: Advanced Fault Localization
/// Extracts file, line, and message for 'Code Peek' UI features.
pub fn extract_error_locations(error_msg: &str) -> Vec<ErrorLocation> {
    let mut locations = Vec::new();
    
    // Pattern: file.c:line:col: error: message
    for line in error_msg.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let file = parts[0].trim();
            if file.ends_with(".c") || file.ends_with(".h") || file.ends_with(".rs") || file.ends_with(".py") {
                if let Ok(line_num) = parts[1].trim().parse::<usize>() {
                    locations.push(ErrorLocation {
                        file: file.to_string(),
                        line: line_num,
                        message: parts[3..].join(":").trim().to_string(),
                    });
                }
            }
        }
    }
    locations
}

pub fn extract_error_files(error_msg: &str) -> Vec<String> {
    extract_error_locations(error_msg).into_iter().map(|l| l.file).collect()
}
