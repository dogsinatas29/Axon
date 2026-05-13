use std::process::Command;
use std::path::Path;

pub enum ValidationMode {
    Incremental,
    Full,
}

pub fn validate(project_root: &str, file_path: &str, mode: ValidationMode) -> anyhow::Result<()> {
    if file_path.ends_with(".rs") {
        validate_rust(project_root, mode)
    } else if file_path.ends_with(".py") {
        validate_python(project_root, file_path)
    } else if file_path.ends_with(".c") {
        validate_c(project_root, file_path)
    } else {
        Ok(())
    }
}

fn validate_c(project_root: &str, file_path: &str) -> anyhow::Result<()> {
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

    // v0.0.28: Stage 4 - Full Build Loop via CMake
    if Path::new(project_root).join("CMakeLists.txt").exists() {
        validate_c_project(project_root)?;
    } else {
        // Fallback: Quick syntax check
        let output = Command::new("gcc")
            .arg("-fsyntax-only")
            .arg(file_path)
            .current_dir(project_root)
            .output()?;
            
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
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

/// v0.0.28: Intelligent Fault Localization
/// Parses compiler/validator stderr to extract offending filenames.
pub fn extract_error_files(error_msg: &str) -> Vec<String> {
    let mut files = std::collections::HashSet::new();
    
    // Pattern 1: GCC/Clang (file.c:line:col: error:)
    for line in error_msg.lines() {
        if let Some(idx) = line.find(':') {
            let potential_file = &line[..idx];
            if potential_file.ends_with(".c") || potential_file.ends_with(".h") || potential_file.ends_with(".rs") || potential_file.ends_with(".py") {
                files.insert(potential_file.to_string());
            }
        }
    }
    
    // Pattern 2: Rust (error: ... --> src/main.rs:10:5)
    for line in error_msg.lines() {
        if let Some(idx) = line.find("--> ") {
            let part = &line[idx + 4..];
            if let Some(colon_idx) = part.find(':') {
                let potential_file = &part[..colon_idx];
                files.insert(potential_file.to_string());
            }
        }
    }

    files.into_iter().collect()
}
