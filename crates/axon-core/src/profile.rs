use axon_ir::schema::Language;
use axon_ir::validator::langs::LanguageValidator;

/// v0.0.31: Language Semantic Profile
/// This trait defines the semantic ownership for a specific language.
/// It will eventually provide language-aware decomposers and emitters.
pub trait LanguageProfile: Send + Sync {
    /// Returns the specific semantic validator for this language.
    fn validator(&self) -> Box<dyn LanguageValidator>;
    
    /// Semantic Firewall: Checks if a given artifact path is legal for this language.
    fn allows_artifact(&self, path: &str) -> bool;
    
    /// Evaluates the task kind corresponding to a given target file for phase separation
    fn determine_task_kind(&self, target_file: &str, is_entrypoint: bool) -> Option<crate::LanguageTaskKind>;

    /// Returns the build command for this language, given sandbox root.
    /// Returns (program, args) pair.
    fn build_command(&self, sandbox_root: &std::path::Path) -> Option<(String, Vec<String>)>;

    /// Returns the run command for this language, given sandbox root and project id.
    fn run_command(&self, sandbox_root: &std::path::Path, project_id: &str) -> Option<(String, Vec<String>)>;
}

pub struct CProfile;
impl LanguageProfile for CProfile {
    fn validator(&self) -> Box<dyn LanguageValidator> {
        Box::new(axon_ir::validator::langs::c::CValidator)
    }

    fn allows_artifact(&self, path: &str) -> bool {
        let p = path.to_lowercase();
        p.ends_with(".c") || p.ends_with(".h") || p.ends_with(".cpp") || p.ends_with(".hpp") || p.contains("cmake") || p.contains("makefile")
    }

    fn determine_task_kind(&self, target_file: &str, is_entrypoint: bool) -> Option<crate::LanguageTaskKind> {
        let p = target_file.to_lowercase();
        if p.ends_with(".h") || p.ends_with(".hpp") {
            Some(crate::LanguageTaskKind::C(crate::CTaskKind::HeaderDecl))
        } else if is_entrypoint || p.contains("main") {
            Some(crate::LanguageTaskKind::C(crate::CTaskKind::Integrator))
        } else {
            Some(crate::LanguageTaskKind::C(crate::CTaskKind::SourceImpl))
        }
    }

    fn build_command(&self, sandbox_root: &std::path::Path) -> Option<(String, Vec<String>)> {
        // cmake --build build (after cmake .. in build/)
        let build_dir = sandbox_root.join("build");
        let _ = std::fs::create_dir_all(&build_dir);
        // Two-step: caller must run cmake .. first, then cmake --build .
        Some(("cmake".to_string(), vec!["--build".to_string(), build_dir.to_string_lossy().to_string()]))
    }

    fn run_command(&self, sandbox_root: &std::path::Path, project_id: &str) -> Option<(String, Vec<String>)> {
        let exe = sandbox_root.join("build").join(project_id);
        Some((exe.to_string_lossy().to_string(), vec![]))
    }
}

pub struct RustProfile;
impl LanguageProfile for RustProfile {
    fn validator(&self) -> Box<dyn LanguageValidator> {
        Box::new(axon_ir::validator::langs::rust::RustValidator)
    }

    fn allows_artifact(&self, path: &str) -> bool {
        let p = path.to_lowercase();
        // Strictly forbid C/C++ artifacts
        if p.ends_with(".h") || p.ends_with(".c") || p.ends_with(".cpp") || p.contains("cmake") {
            return false;
        }
        p.ends_with(".rs") || p.contains("cargo.toml") || p.contains("cargo.lock") || p.ends_with(".md")
    }

    fn determine_task_kind(&self, target_file: &str, is_entrypoint: bool) -> Option<crate::LanguageTaskKind> {
        let p = target_file.to_lowercase();
        if is_entrypoint || p.contains("main.rs") || p.contains("lib.rs") {
            Some(crate::LanguageTaskKind::Rust(crate::RustTaskKind::Integrator))
        } else if p.ends_with("mod.rs") {
            Some(crate::LanguageTaskKind::Rust(crate::RustTaskKind::ModuleDecl))
        } else {
            Some(crate::LanguageTaskKind::Rust(crate::RustTaskKind::ModuleImpl))
        }
    }

    fn build_command(&self, sandbox_root: &std::path::Path) -> Option<(String, Vec<String>)> {
        Some(("cargo".to_string(), vec![
            "build".to_string(),
            "--manifest-path".to_string(),
            sandbox_root.join("Cargo.toml").to_string_lossy().to_string(),
        ]))
    }

    fn run_command(&self, sandbox_root: &std::path::Path, project_id: &str) -> Option<(String, Vec<String>)> {
        Some(("cargo".to_string(), vec![
            "run".to_string(),
            "--manifest-path".to_string(),
            sandbox_root.join("Cargo.toml").to_string_lossy().to_string(),
            "--bin".to_string(),
            project_id.to_string(),
        ]))
    }
}

pub struct PythonProfile;
impl LanguageProfile for PythonProfile {
    fn validator(&self) -> Box<dyn LanguageValidator> {
        Box::new(axon_ir::validator::langs::python::PythonValidator)
    }

    fn allows_artifact(&self, path: &str) -> bool {
        let p = path.to_lowercase();
        if p.ends_with(".h") || p.ends_with(".c") || p.ends_with(".rs") || p.contains("cmake") || p.contains("cargo") {
            return false;
        }
        p.ends_with(".py") || p.contains("requirements.txt") || p.ends_with(".md")
    }

    fn determine_task_kind(&self, _target_file: &str, _is_entrypoint: bool) -> Option<crate::LanguageTaskKind> {
        None
    }

    fn build_command(&self, sandbox_root: &std::path::Path) -> Option<(String, Vec<String>)> {
        // python -m pytest or just python main.py
        Some(("python3".to_string(), vec![
            sandbox_root.join("main.py").to_string_lossy().to_string(),
        ]))
    }

    fn run_command(&self, sandbox_root: &std::path::Path, _project_id: &str) -> Option<(String, Vec<String>)> {
        Some(("python3".to_string(), vec![
            sandbox_root.join("main.py").to_string_lossy().to_string(),
        ]))
    }
}

pub struct LuaProfile;
impl LanguageProfile for LuaProfile {
    fn validator(&self) -> Box<dyn LanguageValidator> {
        Box::new(axon_ir::validator::langs::lua::LuaValidator)
    }

    fn allows_artifact(&self, path: &str) -> bool {
        let p = path.to_lowercase();
        if p.ends_with(".h") || p.ends_with(".c") || p.ends_with(".rs") || p.ends_with(".py") || p.contains("cmake") || p.contains("cargo") {
            return false;
        }
        p.ends_with(".lua") || p.ends_with(".md")
    }

    fn determine_task_kind(&self, _target_file: &str, _is_entrypoint: bool) -> Option<crate::LanguageTaskKind> {
        None
    }

    fn build_command(&self, sandbox_root: &std::path::Path) -> Option<(String, Vec<String>)> {
        Some(("luac".to_string(), vec![
            "-p".to_string(),
            sandbox_root.join("main.lua").to_string_lossy().to_string(),
        ]))
    }

    fn run_command(&self, sandbox_root: &std::path::Path, _project_id: &str) -> Option<(String, Vec<String>)> {
        Some(("lua".to_string(), vec![
            sandbox_root.join("main.lua").to_string_lossy().to_string(),
        ]))
    }
}

/// Profile Registry: Maps the explicit Language enum to its Semantic Profile
pub fn get_profile(lang: Language) -> Box<dyn LanguageProfile> {
    match lang {
        Language::C | Language::Cpp => Box::new(CProfile),
        Language::Rust => Box::new(RustProfile),
        Language::Python => Box::new(PythonProfile),
        Language::Lua => Box::new(LuaProfile),
    }
}
