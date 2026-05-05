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
    } else {
        Ok(())
    }
}

fn validate_rust(project_root: &str, mode: ValidationMode) -> anyhow::Result<()> {
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
        let status = Command::new("cargo")
            .arg("test")
            .current_dir(project_root)
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("Rust runtime validation (tests) failed."));
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
