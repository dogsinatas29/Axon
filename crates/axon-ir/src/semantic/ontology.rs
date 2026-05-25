use crate::schema::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticToken {
    HeaderDecl,
    SourceImpl,
    ModuleDecl,
    ModuleImpl,
    Integrator,
    Package,
    Script,
    CMake,
    Cargo,
    Pip,
    Poetry,
}

impl SemanticToken {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HeaderDecl => "HeaderDecl",
            Self::SourceImpl => "SourceImpl",
            Self::ModuleDecl => "ModuleDecl",
            Self::ModuleImpl => "ModuleImpl",
            Self::Integrator => "Integrator",
            Self::Package => "Package",
            Self::Script => "Script",
            Self::CMake => "cmake",
            Self::Cargo => "cargo",
            Self::Pip => "pip",
            Self::Poetry => "poetry",
        }
    }
}

pub trait SemanticOntology: Send + Sync {
    fn forbidden_vocabulary(&self) -> &[&str];
    fn allowed_task_vocabulary(&self) -> &[&str];
    fn allowed_build_systems(&self) -> &[&str];
    
    // v0.0.31.02: Symbolic tokens for dynamic matching
    fn forbidden_tokens(&self) -> &[SemanticToken];
    fn allowed_tokens(&self) -> &[SemanticToken];
}

pub struct RustOntology;
impl SemanticOntology for RustOntology {
    fn forbidden_vocabulary(&self) -> &[&str] {
        &[
            "HeaderDecl",
            "SourceImpl",
            "Integrator",
            "header",
            "source",
            "CMakeLists.txt",
            "cmake",
            "makefile",
            ".h",
            ".c",
        ]
    }

    fn allowed_task_vocabulary(&self) -> &[&str] {
        &["ModuleDecl", "ModuleImpl", "Integrator"]
    }

    fn allowed_build_systems(&self) -> &[&str] {
        &["cargo"]
    }

    fn forbidden_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::HeaderDecl,
            SemanticToken::SourceImpl,
            SemanticToken::CMake,
        ]
    }

    fn allowed_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::ModuleDecl,
            SemanticToken::ModuleImpl,
            SemanticToken::Integrator,
            SemanticToken::Cargo,
        ]
    }
}

pub struct COntology;
impl SemanticOntology for COntology {
    fn forbidden_vocabulary(&self) -> &[&str] {
        &[]
    }

    fn allowed_task_vocabulary(&self) -> &[&str] {
        &["HeaderDecl", "SourceImpl", "Integrator"]
    }

    fn allowed_build_systems(&self) -> &[&str] {
        &["cmake", "make", "gcc", "clang"]
    }

    fn forbidden_tokens(&self) -> &[SemanticToken] {
        &[]
    }

    fn allowed_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::HeaderDecl,
            SemanticToken::SourceImpl,
            SemanticToken::Integrator,
            SemanticToken::CMake,
        ]
    }
}

pub struct CppOntology;
impl SemanticOntology for CppOntology {
    fn forbidden_vocabulary(&self) -> &[&str] {
        &[]
    }

    fn allowed_task_vocabulary(&self) -> &[&str] {
        &["HeaderDecl", "SourceImpl", "Integrator"]
    }

    fn allowed_build_systems(&self) -> &[&str] {
        &["cmake", "make", "gcc", "clang", "g++", "clang++"]
    }

    fn forbidden_tokens(&self) -> &[SemanticToken] {
        &[]
    }

    fn allowed_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::HeaderDecl,
            SemanticToken::SourceImpl,
            SemanticToken::Integrator,
            SemanticToken::CMake,
        ]
    }
}

pub struct PythonOntology;
impl SemanticOntology for PythonOntology {
    fn forbidden_vocabulary(&self) -> &[&str] {
        &[
            "HeaderDecl",
            "SourceImpl",
            "Integrator",
            "Cargo.toml",
            "cargo",
            "cmake",
            "CMakeLists.txt",
            "mod.rs",
            "lib.rs",
        ]
    }

    fn allowed_task_vocabulary(&self) -> &[&str] {
        &["Module", "Package", "Script"]
    }

    fn allowed_build_systems(&self) -> &[&str] {
        &["python", "pip", "poetry", "pytest"]
    }

    fn forbidden_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::HeaderDecl,
            SemanticToken::SourceImpl,
            SemanticToken::Integrator,
            SemanticToken::CMake,
            SemanticToken::Cargo,
        ]
    }

    fn allowed_tokens(&self) -> &[SemanticToken] {
        &[
            SemanticToken::Package,
            SemanticToken::Script,
            SemanticToken::Pip,
            SemanticToken::Poetry,
        ]
    }
}

pub fn get_ontology(language: Language) -> Box<dyn SemanticOntology> {
    match language {
        Language::C => Box::new(COntology),
        Language::Cpp => Box::new(CppOntology),
        Language::Rust => Box::new(RustOntology),
        Language::Python => Box::new(PythonOntology),
    }
}