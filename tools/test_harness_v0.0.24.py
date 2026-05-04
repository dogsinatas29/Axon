#!/usr/bin/env python3
# encoding: utf-8
import sys
import os
import tempfile
import shutil

# Import the harness functions
sys.path.append(os.path.dirname(__file__))
from axon_execution_harness import strip_markdown, verify_file_integrity

def test_markdown_stripping():
    print("Running Test: Markdown Stripping...")
    contaminated_code = """```rust
fn test() {
    println!("Hello");
}
```"""
    expected_clean = """
fn test() {
    println!("Hello");
}
"""
    result = strip_markdown(contaminated_code)
    # Basic check: should not contain backticks
    if "```" in result:
        print("❌ FAILED: Markdown fences still present after stripping.")
        return False
    print("✅ PASSED: Markdown stripping successful.")
    return True

def test_markdown_rejection():
    print("Running Test: Markdown Rejection in Integrity Audit...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        fpath = os.path.join(tmp_dir, "test.rs")
        code = "```rust\nfn fail() {}\n```"
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(code)
            
        # verify_file_integrity returns a list of errors
        errors = verify_file_integrity(tmp_dir, ["test.rs"], target_file="test.rs")
        
        if any("Markdown pollution detected" in e for e in errors):
            print("✅ PASSED: Markdown pollution correctly detected and rejected.")
            return True
        else:
            print("❌ FAILED: Markdown pollution was NOT detected.")
            print(f"Errors found: {errors}")
            return False

def test_clean_code_pass():
    print("Running Test: Clean Code Pass...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        fpath = os.path.join(tmp_dir, "clean.rs")
        code = "fn success() {\n    // This is valid code\n    let x = 10;\n}"
        # Add some padding to pass the 120 byte logic density check if needed
        code += "\n" + "// " * 50 
        
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(code)
            
        errors = verify_file_integrity(tmp_dir, ["clean.rs"], target_file="clean.rs")
        
        if not errors:
            print("✅ PASSED: Clean code passed integrity audit.")
            return True
        else:
            print("❌ FAILED: Clean code was rejected.")
            print(f"Errors found: {errors}")
            return False

def test_scope_violation():
    print("Running Test: Scope Violation Detection...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        fpath = os.path.join(tmp_dir, "violation.rs")
        # Pattern like 'mod other;' should be rejected
        code = "mod other_component;\nfn test() {}"
        code += "\n" + "// " * 50
        
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(code)
            
        errors = verify_file_integrity(tmp_dir, ["violation.rs"], target_file="violation.rs")
        
        if any("Scope violation detected" in e for e in errors):
            print("✅ PASSED: Scope violation (mod) correctly detected.")
            return True
        else:
            print("❌ FAILED: Scope violation was NOT detected.")
            return False

def test_multi_file_rejection():
    print("Running Test: Multi-file Write Gate Enforcement...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Simulate agent trying to write two files when only one is expected
        files = ["target.rs", "malicious.rs"]
        for f in files:
            with open(os.path.join(tmp_dir, f), "w") as f_obj:
                f_obj.write("// " * 50)
        
        errors = verify_file_integrity(tmp_dir, files, target_file="target.rs")
        
        if any("SCOPE_VIOLATION: Multi-file modification attempt" in e for e in errors):
            print("✅ PASSED: Unauthorized multi-file write correctly rejected.")
            return True
        else:
            print("❌ FAILED: Multi-file write was NOT rejected.")
            return False

def test_system_access_violation():
    print("Running Test: System Access Violation (std::fs)...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        fpath = os.path.join(tmp_dir, "sys.rs")
        code = "fn hack() {\n    std::fs::remove_file(\"important.db\");\n}"
        code += "\n" + "// " * 50
        with open(fpath, "w", encoding="utf-8") as f:
            f.write(code)
            
        errors = verify_file_integrity(tmp_dir, ["sys.rs"], target_file="sys.rs")
        if any("Scope violation detected" in e for e in errors):
            print("✅ PASSED: System access (std::fs) correctly detected.")
            return True
        else:
            print("❌ FAILED: System access was NOT detected.")
            return False

def test_target_mismatch_rejection():
    print("Running Test: Target Mismatch Rejection...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Expected target.rs, but agent gave wrong_target.rs
        files = ["wrong_target.rs"]
        with open(os.path.join(tmp_dir, files[0]), "w") as f:
            f.write("// " * 50)
            
        errors = verify_file_integrity(tmp_dir, files, target_file="target.rs")
        if any("SCOPE_VIOLATION: Target mismatch" in e for e in errors):
            print("✅ PASSED: Target filename mismatch correctly rejected.")
            return True
        else:
            print("❌ FAILED: Target mismatch was NOT rejected.")
            return False

if __name__ == "__main__":
    results = [
        test_markdown_stripping(),
        test_markdown_rejection(),
        test_clean_code_pass(),
        test_scope_violation(),
        test_multi_file_rejection(),
        test_system_access_violation(),
        test_target_mismatch_rejection()
    ]
    
    if all(results):
        print("\n🏆 v0.0.24 BOOTSTRAP VERIFICATION SUCCESSFUL")
        sys.exit(0)
    else:
        print("\n❌ v0.0.24 BOOTSTRAP VERIFICATION FAILED")
        sys.exit(1)
