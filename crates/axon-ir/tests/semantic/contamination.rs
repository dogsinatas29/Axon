use axon_ir::semantic::{SemanticSpec, SpecSemanticValidator};

#[test]
fn rust_rejects_header_decl() {
    let json = r#"{
        "language": "rust",
        "components": [{"name": "foo", "kind": "HeaderDecl"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Rust should reject HeaderDecl vocabulary");
}

#[test]
fn rust_rejects_cmake() {
    let json = r#"{
        "language": "rust",
        "build_system": "cmake",
        "components": [{"name": "main", "file": "main.rs"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Rust should reject CMake build system");
}

#[test]
fn rust_rejects_c_header() {
    let json = r#"{
        "language": "rust",
        "components": [{"name": "header", "file": "include/foo.h"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Rust should reject .h files");
}

#[test]
fn rust_allows_module_decl() {
    let json = r#"{
        "language": "rust",
        "components": [{"name": "utils", "kind": "ModuleDecl"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "Rust should allow ModuleDecl vocabulary");
}

#[test]
fn rust_allows_cargo() {
    let json = r#"{
        "language": "rust",
        "build_system": "cargo",
        "components": [{"name": "main", "file": "main.rs"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "Rust should allow cargo build system");
}

#[test]
fn c_allows_header_decl() {
    let json = r#"{
        "language": "c",
        "components": [{"name": "foo", "kind": "HeaderDecl"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "C should allow HeaderDecl vocabulary (permissive baseline)");
}

#[test]
fn c_allows_cmake() {
    let json = r#"{
        "language": "c",
        "build_system": "cmake",
        "components": [{"name": "main", "file": "main.c"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "C should allow cmake build system");
}

#[test]
fn c_allows_c_header() {
    let json = r#"{
        "language": "c",
        "components": [{"name": "header", "file": "include/foo.h"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "C should allow .h files");
}

#[test]
fn python_rejects_cargo() {
    let json = r#"{
        "language": "python",
        "build_system": "cargo",
        "components": [{"name": "main", "file": "main.py"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Python should reject cargo build system");
}

#[test]
fn python_rejects_cmake() {
    let json = r#"{
        "language": "python",
        "build_system": "cmake",
        "components": [{"name": "main", "file": "main.py"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Python should reject cmake build system");
}

#[test]
fn python_rejects_header_decl() {
    let json = r#"{
        "language": "python",
        "components": [{"name": "foo", "kind": "HeaderDecl"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Python should reject C vocabulary");
}

#[test]
fn python_rejects_mod_rs() {
    let json = r#"{
        "language": "python",
        "components": [{"name": "module", "file": "mod.rs"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_err(), "Python should reject mod.rs");
}

#[test]
fn python_allows_python() {
    let json = r#"{
        "language": "python",
        "build_system": "python",
        "components": [{"name": "main", "file": "main.py"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "Python should allow python build system");
}

#[test]
fn cpp_allows_cmake() {
    let json = r#"{
        "language": "cpp",
        "build_system": "cmake",
        "components": [{"name": "main", "file": "main.cpp"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "Cpp should allow cmake build system");
}

#[test]
fn cpp_allows_header() {
    let json = r#"{
        "language": "cpp",
        "components": [{"name": "foo", "kind": "HeaderDecl"}]
    }"#;
    let spec = SemanticSpec::from_llm_json(json).unwrap();
    let result = SpecSemanticValidator::validate(&spec);
    assert!(result.is_ok(), "Cpp should allow HeaderDecl vocabulary");
}