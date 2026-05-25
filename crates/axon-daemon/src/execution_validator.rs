use std::path::Path;
use std::process::Command;
use axon_ir::ProjectIR;
use axon_ir_validator::PlatformValidator;

pub enum ValidationMode {
    Incremental,
    Full,
}

/// v0.0.32: Compile failure classification for accurate blame attribution
#[derive(Debug, Clone, PartialEq)]
pub enum CompileFailureKind {
    /// Syntax error in a specific file - worker responsible
    SyntaxError,
    /// Type error - worker responsible
    TypeError,
    /// Link error - worker responsible
    LinkError,
    /// Missing entrypoint (main.rs/lib.rs) - orchestration issue, NO blame
    MissingEntrypoint,
    /// Missing module (unresolved import) - orchestration issue, NO blame
    MissingModule,
    /// Missing dependency/crate - orchestration issue, NO blame
    MissingDependency,
    /// Project not fully bootstrapped - orchestration issue, NO blame
    ProjectIncomplete,
    /// Unknown error - conservative: don't blame worker
    Unknown,
}

impl CompileFailureKind {
    pub fn should_quarantine(&self) -> bool {
        matches!(
            self,
            CompileFailureKind::SyntaxError
                | CompileFailureKind::TypeError
                | CompileFailureKind::LinkError
        )
    }

    pub fn is_orchestration_issue(&self) -> bool {
        matches!(
            self,
            CompileFailureKind::MissingEntrypoint
                | CompileFailureKind::MissingModule
                | CompileFailureKind::MissingDependency
                | CompileFailureKind::ProjectIncomplete
                | CompileFailureKind::Unknown
        )
    }
}

/// v0.0.32: Classify compile failure from stderr
pub fn classify_compile_failure(stderr: &str) -> CompileFailureKind {
    let stderr_lower = stderr.to_lowercase();

    // Missing entrypoint patterns
    if stderr.contains("can't find bin")
        || stderr.contains("file not found as binary")
        || stderr.contains("could not find bin")
        || (stderr.contains("main") && stderr.contains("not found"))
    {
        return CompileFailureKind::MissingEntrypoint;
    }

    // Missing module patterns
    if stderr.contains("cannot find component")
        || stderr.contains("unresolved import")
        || stderr.contains("use of undeclared")
        || stderr.contains("use of undeclared crate")
        || stderr_lower.contains("module not found")
        || stderr_lower.contains("file not found for module")
    {
        return CompileFailureKind::MissingModule;
    }

    // Missing dependency patterns
    if stderr.contains("can't find crate")
        || stderr.contains("cannot find external dependency")
        || stderr.contains("no such file or directory: '")
        || stderr.contains("external crate")
    {
        return CompileFailureKind::MissingDependency;
    }

    // Project incomplete patterns
    if stderr.contains("no such file or directory: src/main.rs")
        || stderr.contains("no manifest")
        || stderr.contains("src/main.rs")
        || (stderr.contains("could not compile") && stderr.contains("src/"))
    {
        // Check if it's specifically about missing files (not syntax errors)
        if stderr.contains("No such file or directory")
            || stderr.contains("file not found")
            || stderr.contains("not found in")
        {
            return CompileFailureKind::ProjectIncomplete;
        }
    }

    // Actual syntax/type errors - these ARE worker responsible
    if stderr.contains("expected item, found")
        || stderr.contains("expected expression")
        || stderr.contains("expected `;`")
        || stderr.contains("expected `}`")
        || stderr.contains("expected `fn`")
        || stderr.contains("expected struct")
        || stderr.contains("expected `::`")
        || stderr.contains("expected identifier")
        || stderr.contains("expected module")
    {
        return CompileFailureKind::SyntaxError;
    }

    // Type errors
    if stderr.contains("mismatched types")
        || stderr.contains("type mismatch")
        || stderr.contains("expected `")
        && stderr.contains("` but found")
        || stderr.contains("cannot return value referencing")
        || stderr.contains("lifetime")
    {
        return CompileFailureKind::TypeError;
    }

    // Link errors
    if stderr.contains("undefined reference")
        || stderr.contains("multiple definition")
        || stderr.contains("linker")
    {
        return CompileFailureKind::LinkError;
    }

    CompileFailureKind::Unknown
}

/// v0.0.32: Bootstrap completion check for Rust projects
/// Returns (is_complete, missing_files)
pub fn project_bootstrap_complete(
    project_root: &str,
    ir: Option<&ProjectIR>,
) -> (bool, Vec<String>) {
    let root = Path::new(project_root);
    let mut missing = Vec::new();

    // Minimum Rust project requirements
    if !root.join("Cargo.toml").exists() {
        missing.push("Cargo.toml".to_string());
    }
    if !root.join("src/main.rs").exists() && !root.join("src/lib.rs").exists() {
        missing.push("src/main.rs or src/lib.rs".to_string());
    }

    // IR-based component closure check (v0.0.32)
    // Only require files that are actually part of the IR
    if let Some(ir_data) = ir {
        // Check for entrypoint existence
        let has_entrypoint = ir_data.components.values().any(|c| c.is_entrypoint);
        if has_entrypoint {
            // If IR defines an entrypoint, it must exist physically
            let entrypoint_exists = ir_data
                .components
                .iter()
                .any(|(path, comp)| {
                    if comp.is_entrypoint {
                        root.join(path).exists()
                    } else {
                        false
                    }
                });
            if !entrypoint_exists && !missing.is_empty() {
                // Already have main.rs in missing, this is additional info
            }
        }

        // Check critical modules from IR
        for (component_path, _) in &ir_data.components {
            let physical_path = root.join(component_path);
            // Only check if the path suggests a required file
            // Skip if it's a subdirectory structure that doesn't require physical file
            if component_path.ends_with(".rs") && !physical_path.exists() {
                // File in IR but not materialized yet - project incomplete
                missing.push(component_path.clone());
            }
        }
    }

    (missing.is_empty(), missing)
}

pub fn validate(project_root: &str, file_path: &str, mode: ValidationMode, ir: Option<&ProjectIR>) -> anyhow::Result<()> {
    // Platform-Specific Source Validation (Win32 OS Runtime Contract)
    if let Some(ir_data) = ir {
        if ir_data.platform == axon_ir::Platform::Win32 {
            let path = Path::new(project_root).join(file_path);
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let platform_val = PlatformValidator::new();
                let is_entrypoint = ir_data.components.get(file_path).map_or(false, |c| c.is_entrypoint);
                platform_val.validate_source_code(file_path, &content, is_entrypoint)?;
            }
        }
    }

    // v0.0.29: Structural Validation (Semantic completeness & safety)
    analyze_structural_integrity(project_root, file_path, ir)?;

    if file_path.ends_with(".rs") {
        validate_rust(project_root, mode, ir)
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
        // F. Semantic Invariant & Strict Spec Constraint Enforcement (v0.0.31 Hardened v21)
        if file_path.contains("calculate.rs") {
            if !content.contains("chrono") || !content.contains("Local::now()") {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: 'calculate.rs' must strictly enforce the chrono invariant. Hardcoded date values are forbidden. Please use 'chrono::Local::now()' to determine the current year dynamically.",
                ));
            }
            if content.contains("let current_year = 2023;") || content.contains("current_year = 2023") || content.contains("let current_year = 2024;") {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: Hardcoded temporal state detected in 'calculate.rs'. Do not hardcode current_year (e.g. 2023, 2024). You must bind to 'chrono::Local::now()' dynamically.",
                ));
            }
        }

        if file_path.contains("validation.rs") {
            if content.contains("year < -9999") || content.contains("year > 9999") {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: Hardcoded stub boundaries (-9999..9999) detected in 'validation.rs'. You must dynamically validate the year based on: 'Year > current_year' and 'Year < current_year - 120'.",
                ));
            }
            if content.contains("fn validate_day") && !content.contains("from_ymd") && !content.contains("NaiveDate") {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: 'validate_day' in 'validation.rs' must perform robust calendar validation (e.g. leap year check, 2/30 prevention) using chrono's 'NaiveDate::from_ymd_opt'.",
                ));
            }
        }

        if file_path.contains("db.rs") {
            if !content.contains("rusqlite") || !content.contains("user_records") {
                return Err(anyhow::anyhow!(
                    "SEMANTIC VIOLATION: 'db.rs' must strictly implement the SQLite3 data layer using the 'rusqlite' crate. Stub placeholders are forbidden. Table name must be 'user_records' with fields: name, birth_year, birth_month, birth_day.",
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
        validate_c_project(project_root, _ir)?;
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

fn validate_c_project(project_root: &str, ir: Option<&ProjectIR>) -> anyhow::Result<()> {
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

    // 3. Win32 PE Subsystem Verification (OS Runtime Contract)
    if let Some(ir_data) = ir {
        if ir_data.platform == axon_ir::Platform::Win32 {
            let platform_val = PlatformValidator::new();
            let mut validated_at_least_one = false;
            if let Ok(entries) = std::fs::read_dir(&build_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let is_potential_exe = path.extension().map_or(true, |ext| ext == "exe");
                        if is_potential_exe {
                            if let Ok(bytes) = std::fs::read(&path) {
                                if bytes.len() >= 64 {
                                    let pe_offset = u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;
                                    if bytes.len() >= pe_offset + 4 && bytes[pe_offset] == b'P' && bytes[pe_offset+1] == b'E' {
                                        if let Err(e) = platform_val.validate_binary_subsystem(path.to_str().unwrap_or("")) {
                                            return Err(e);
                                        }
                                        validated_at_least_one = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !validated_at_least_one {
                tracing::warn!("⚠️ [PLATFORM_VALIDATOR] Win32 GUI executable not validated or not found in build dir.");
            } else {
                tracing::info!("✅ [PLATFORM_VALIDATOR] Successfully validated Win32 subsystem for GUI executables.");
            }
        }
    }
    
    Ok(())
}

fn validate_rust(project_root: &str, mode: ValidationMode, ir: Option<&ProjectIR>) -> anyhow::Result<()> {
    // v0.0.32: Bootstrap barrier & Deferred Cargo Check - check minimum project requirements
    let (is_bootstrap_complete, missing) = project_bootstrap_complete(project_root, ir);
    if !is_bootstrap_complete {
        tracing::info!(
            "🍃 [BOOTSTRAP_DEFERRAL] Skipping cargo check. Project incomplete (missing: {}). Temporarily passing validation.",
            missing.join(", ")
        );
        return Ok(()); // Defer cargo check until all required files exist
    }

    // Phase 1: Try incremental (Native cargo check)
    if let ValidationMode::Full = mode {
        let _ = Command::new("cargo")
            .arg("clean")
            .current_dir(project_root)
            .status();
    }

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // v0.0.32: Classify failure before deciding action
        let failure_kind = classify_compile_failure(&stderr);

        // Log the classification
        tracing::warn!(
            "⚠️ [COMPILE_FAIL:{:?}] Cargo check failed.\n  stderr preview: {}",
            failure_kind,
            stderr.chars().take(200).collect::<String>()
        );

        // v0.0.32: Project incomplete = orchestration issue, NOT worker fault
        if failure_kind.is_orchestration_issue() {
            return Err(anyhow::anyhow!(
                "ORCHESTRATION_ISSUE:{:?} - {}",
                failure_kind,
                stderr.chars().take(300).collect::<String>()
            ));
        }

        // Actual code errors - retry with full clean
        match mode {
            ValidationMode::Incremental => {
                tracing::warn!("⚠️ [INCREMENTAL_FAIL] Cargo check failed. Retrying with FULL CLEAN to verify...");
                validate_rust(project_root, ValidationMode::Full, ir)
            }
            ValidationMode::Full => {
                Err(anyhow::anyhow!(
                    "CODE_ERROR:{:?} - {}",
                    failure_kind,
                    stderr.chars().take(500).collect::<String>()
                ))
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

/// v0.0.29: Advanced Fault Localization & Compile Attribution Engine [HARDENED]
/// Extracts file, line, and message for 'Code Peek' UI features across C, Rust, and Python compilers.
pub fn extract_error_locations(error_msg: &str) -> Vec<ErrorLocation> {
    let mut locations = Vec::new();
    
    for line in error_msg.lines() {
        let trimmed = line.trim();
        
        // 1. Rust Cargo format: "--> src/validation.rs:11:5"
        if trimmed.starts_with("-->") {
            let path_part = trimmed.trim_start_matches("-->").trim();
            let parts: Vec<&str> = path_part.split(':').collect();
            if parts.len() >= 2 {
                let file = parts[0].trim().to_string();
                if let Ok(line_num) = parts[1].trim().parse::<usize>() {
                    locations.push(ErrorLocation {
                        file,
                        line: line_num,
                        message: "Rust compiler check diagnostic".to_string(),
                    });
                    continue;
                }
            }
        }

        // 2. Python standard stack trace: "File \"src/validation.py\", line 11"
        if trimmed.starts_with("File \"") {
            if let Some(end_quote_idx) = trimmed[6..].find('"') {
                let file = trimmed[6..6 + end_quote_idx].to_string();
                if let Some(line_word_idx) = trimmed.find("line ") {
                    let line_part = &trimmed[line_word_idx + 5..];
                    let line_num_str: String = line_part.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(line_num) = line_num_str.parse::<usize>() {
                        locations.push(ErrorLocation {
                            file,
                            line: line_num,
                            message: "Python interpreter diagnostic".to_string(),
                        });
                        continue;
                    }
                }
            }
        }
        
        // 3. C GCC format: "file.c:line:col: error: message"
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
    let mut files = Vec::new();
    for loc in extract_error_locations(error_msg) {
        let f = loc.file.clone();
        // Canonicalize file path to match short target filenames
        let canonical = f.trim_start_matches("./").to_string();
        if !files.contains(&canonical) {
            files.push(canonical);
        }
    }
    files
}

pub fn extract_undefined_symbols(error_msg: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in error_msg.lines() {
        let trimmed = line.trim();
        // GCC/Clang: "undefined reference to `my_func'"
        if let Some(idx) = trimmed.find("undefined reference to `") {
            let start = idx + 24;
            if let Some(end) = trimmed[start..].find('\'').or_else(|| trimmed[start..].find('`')) {
                symbols.push(trimmed[start..start + end].to_string());
            }
        } else if let Some(idx) = trimmed.find("undefined reference to ") {
            // "undefined reference to my_func"
            let rest = &trimmed[idx + 23..];
            let sym = rest.split_whitespace().next().unwrap_or("").trim_matches('\'').trim_matches('`').trim_matches('"');
            if !sym.is_empty() {
                symbols.push(sym.to_string());
            }
        }
        
        // Rust: "cannot find function `my_func` in this scope"
        if let Some(idx) = trimmed.find("cannot find function `") {
            let start = idx + 22;
            if let Some(end) = trimmed[start..].find('`') {
                symbols.push(trimmed[start..start + end].to_string());
            }
        }
        // Rust: "cannot find struct, variant or union type `MyStruct`"
        if let Some(idx) = trimmed.find("cannot find struct, variant or union type `") {
            let start = idx + 43;
            if let Some(end) = trimmed[start..].find('`') {
                symbols.push(trimmed[start..start + end].to_string());
            }
        }
        // Rust: "use of undeclared crate or module `my_mod`"
        if let Some(idx) = trimmed.find("use of undeclared crate or module `") {
            let start = idx + 35;
            if let Some(end) = trimmed[start..].find('`') {
                symbols.push(trimmed[start..start + end].to_string());
            }
        }
    }
    symbols.sort();
    symbols.dedup();
    symbols
}
