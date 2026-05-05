#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Execution Harness v2 (Plugin-based Architecture)
Provides language-aware validation, structured error reporting, and plugin registry.
"""
import subprocess
import tempfile
import os
import shutil
import sys
import argparse
import json
import re

# =============================================================================
# 1. Base Structures & Interfaces
# =============================================================================

class ValidationResult:
    def __init__(self, ok: bool, stage: str, language: str, detail: str, stdout: str = "", stderr: str = ""):
        self.ok = ok
        self.stage = stage
        self.language = language
        self.detail = detail
        self.stdout = stdout
        self.stderr = stderr

    def to_dict(self):
        return {
            "status": "success" if self.ok else "fail",
            "stage": self.stage,
            "language": self.language,
            "error": self.detail if not self.ok else None,
            "stdout": self.stdout,
            "stderr": self.stderr
        }

class BaseValidator:
    def compile(self, project_path: str, entry_file: str) -> ValidationResult:
        return ValidationResult(True, "compile", "base", "No compilation needed")

    def run(self, project_path: str, entry_file: str, timeout: int = 10) -> ValidationResult:
        return ValidationResult(True, "run", "base", "No execution logic")

    def validate(self, project_path: str, entry_file: str, timeout: int = 10) -> ValidationResult:
        # Phase A: Physical Check (Compile/Check)
        res = self.compile(project_path, entry_file)
        if not res.ok:
            return res
        
        # Phase B: Runtime Check (Run)
        return self.run(project_path, entry_file, timeout)

# =============================================================================
# 2. Language Specific Validators
# =============================================================================

class RustValidator(BaseValidator):
    def compile(self, project_path: str, entry_file: str) -> ValidationResult:
        try:
            # v0.0.25: cargo check is the SSOT for Rust physical integrity
            cmd = ["cargo", "check"]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=project_path, timeout=60)
            
            if result.returncode != 0:
                return ValidationResult(False, "compile", "rust", result.stderr, stdout=result.stdout, stderr=result.stderr)
            return ValidationResult(True, "compile", "rust", "Cargo check passed")
        except Exception as e:
            return ValidationResult(False, "compile", "rust", str(e))

    def run(self, project_path: str, entry_file: str, timeout: int = 10) -> ValidationResult:
        try:
            # We only run if main.rs or a binary exists, otherwise skip (library mode)
            if not os.path.exists(os.path.join(project_path, "src", "main.rs")) and not os.path.exists(os.path.join(project_path, "main.rs")):
                return ValidationResult(True, "run", "rust", "Skipping run for library component")
                
            cmd = ["cargo", "run", "--quiet"]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=project_path, timeout=timeout)
            
            if result.returncode != 0:
                return ValidationResult(False, "run", "rust", result.stderr, stdout=result.stdout, stderr=result.stderr)
            return ValidationResult(True, "run", "rust", "Cargo run successful", stdout=result.stdout)
        except Exception as e:
            return ValidationResult(False, "run", "rust", str(e))

class PythonValidator(BaseValidator):
    def compile(self, project_path: str, entry_file: str) -> ValidationResult:
        try:
            cmd = [sys.executable, "-m", "py_compile", entry_file]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=project_path, timeout=15)
            
            if result.returncode != 0:
                return ValidationResult(False, "compile", "python", result.stderr, stdout=result.stdout, stderr=result.stderr)
            return ValidationResult(True, "compile", "python", "Python syntax check passed")
        except Exception as e:
            return ValidationResult(False, "compile", "python", str(e))

    def run(self, project_path: str, entry_file: str, timeout: int = 10) -> ValidationResult:
        try:
            cmd = [sys.executable, entry_file]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=project_path, timeout=timeout)
            
            if result.returncode != 0:
                return ValidationResult(False, "run", "python", result.stderr, stdout=result.stdout, stderr=result.stderr)
            return ValidationResult(True, "run", "python", "Python execution successful", stdout=result.stdout)
        except Exception as e:
            return ValidationResult(False, "run", "python", str(e))

# =============================================================================
# 3. Detector & Registry
# =============================================================================

class LanguageDetector:
    @staticmethod
    def detect(project_path: str, entry_file: str) -> str:
        if os.path.exists(os.path.join(project_path, "Cargo.toml")):
            return "rust"
        if entry_file.endswith(".rs"):
            return "rust"
        if entry_file.endswith(".py"):
            return "python"
        
        # Scan files
        for f in os.listdir(project_path):
            if f.endswith(".py"): return "python"
            if f.endswith(".rs"): return "rust"
            
        return "unknown"

VALIDATORS = {
    "rust": RustValidator(),
    "python": PythonValidator()
}

# =============================================================================
# 4. Integrity Checks (F1, F2)
# =============================================================================

FORBIDDEN_FILES = ["architecture.md", "mile_stone/", "release_note/", ".gemini/", "axon_execution_harness.py"]

def verify_file_integrity(target_dir: str, expected_files: list, target_file: str = None):
    errors = []
    for fname in expected_files:
        fpath = os.path.join(target_dir, fname)
        if not os.path.exists(fpath):
            errors.append(f"F1: Missing file '{fname}'")
            continue
        
        try:
            size = os.path.getsize(fpath)
            if size == 0:
                errors.append(f"F2: File '{fname}' is empty")
            
            with open(fpath, 'r', encoding='utf-8') as f:
                content = f.read()
                if "TODO" in content or "Implementation pending" in content:
                    errors.append(f"F2.1: Stub detected in '{fname}'")
                if "```" in content:
                    errors.append(f"F2.5: Markdown pollution in '{fname}'")
        except Exception as e:
            errors.append(f"F2: Integrity error on '{fname}': {e}")
            
    return errors

# =============================================================================
# 5. Execution Pipeline
# =============================================================================

def execution_harness(project_root: str, file_map: dict, entry_point: str = "main.py", target_file: str = None):
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Prepare sandbox
        if os.path.exists(project_root):
            for root, dirs, files in os.walk(project_root):
                dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ["target", "crates", "tools"]]
                rel_path = os.path.relpath(root, project_root)
                target_dir = os.path.join(tmp_dir, rel_path)
                os.makedirs(target_dir, exist_ok=True)
                for f in files:
                    shutil.copy2(os.path.join(root, f), os.path.join(target_dir, f))

        for fname, code in file_map.items():
            fpath = os.path.join(tmp_dir, fname)
            os.makedirs(os.path.dirname(fpath), exist_ok=True)
            clean_code = "\n".join([l for l in code.splitlines() if not l.strip().startswith("```")])
            with open(fpath, "w", encoding="utf-8") as f:
                f.write(clean_code)

        # Step 1: Static Integrity
        integrity_errors = verify_file_integrity(tmp_dir, list(file_map.keys()), target_file)
        if integrity_errors:
            return False, "\n".join(integrity_errors)

        # Step 2: Language-aware Validation
        lang = LanguageDetector.detect(tmp_dir, entry_point)
        if lang not in VALIDATORS:
            return False, f"Unsupported language: {lang}"
        
        validator = VALIDATORS[lang]
        res = validator.validate(tmp_dir, entry_point)
        
        if not res.ok:
            report = res.to_dict()
            return False, f"[{report['language'].upper()}_{report['stage'].upper()}_FAIL]\n{report['error']}"
        
        return True, res.stdout

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="AXON Execution Harness v2")
    parser.add_argument("--project-root", required=True)
    parser.add_argument("--files-json", required=True)
    parser.add_argument("--entry", default="main.py")
    parser.add_argument("--target-file")
    parser.add_argument("--commit", action="store_true")
    
    args = parser.parse_args()
    
    try:
        with open(args.files_json, 'r', encoding='utf-8') as f:
            file_map = json.load(f)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
        
    success, output = execution_harness(args.project_root, file_map, args.entry, args.target_file)
    
    if success:
        if args.commit:
            for fname, code in file_map.items():
                fpath = os.path.join(args.project_root, fname)
                os.makedirs(os.path.dirname(fpath), exist_ok=True)
                clean_code = "\n".join([l for l in code.splitlines() if not l.strip().startswith("```")])
                with open(fpath, "w", encoding="utf-8") as f:
                    f.write(clean_code)
            print("<<<<HARNESS_SUCCESS_COMMITTED>>>>")
        else:
            print("<<<<HARNESS_SUCCESS_VALIDATED>>>>")
        print(output)
        sys.exit(0)
    else:
        print(f"ERROR: {output}", file=sys.stderr)
        sys.exit(1)
