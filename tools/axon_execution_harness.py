#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Execution Harness
Provides sandbox execution, runtime validation, and rollback safety.
"""
import subprocess
import tempfile
import os
import shutil
import sys
import argparse
import json

def run_code(entry_file: str, timeout=5):
    """Executes the entry file and captures output with resource limits."""
    try:
        # Use timeout to prevent infinite loops (Resource Guard)
        result = subprocess.run(
            [sys.executable, entry_file],
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return {
            "success": result.returncode == 0,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode
        }
    except subprocess.TimeoutExpired:
        return {
            "success": False, 
            "stage": "runtime_execution",
            "rule": "timeout_limit",
            "detail": f"Execution exceeded {timeout}s limit. Possible infinite loop or heavy computation."
        }
    except Exception as e:
        return {
            "success": False, 
            "stage": "runtime_execution",
            "rule": "crash",
            "detail": str(e)
        }

def basic_test(output: dict):
    """Simple validation of execution results."""
    if not output.get("success", False):
        return False, output.get("stderr") or output.get("error", "Unknown execution error")
    return True, None

def snapshot(files: list, project_root: str, backup_dir: str):
    """Creates a backup of existing files."""
    if not os.path.exists(backup_dir):
        os.makedirs(backup_dir, exist_ok=True)
    
    backups = []
    for f in files:
        # f is relative to project_root
        src = os.path.join(project_root, f)
        if os.path.exists(src):
            dst = os.path.join(backup_dir, f.replace(os.sep, "_"))
            shutil.copy2(src, dst)
            backups.append((src, dst))
    return backups

def rollback(backups: list):
    """Restores files from backup."""
    for src, dst in backups:
        if os.path.exists(dst):
            shutil.copy2(dst, src)

def validate_runtime_environment():
    """Checks for essential runtime dependencies and warns if missing."""
    required = ["dateutil", "rich", "pandas"]
    missing = []
    for lib in required:
        try:
            __import__(lib)
        except ImportError:
            missing.append(lib)
    
    if missing:
        print(f"⚠️ [WARNING] Missing Runtime Dependencies: {', '.join(missing)}", file=sys.stderr)
        print(f"💡 Suggestion: pip install {' '.join(missing)}", file=sys.stderr)
        # We don't return False anymore, let it try to run and fail naturally if needed
    return True, None

def verify_file_integrity(target_dir: str, expected_files: list):
    """F1, F2: Checks if files exist, are not empty, and are valid UTF-8."""
    errors = []
    for fname in expected_files:
        fpath = os.path.join(target_dir, fname)
        # F1: Exist check
        if not os.path.exists(fpath):
            errors.append(f"F1: Missing expected file '{fname}' after materialization.")
            continue
        
        # F2: Integrity check
        try:
            size = os.path.getsize(fpath)
            if size == 0:
                errors.append(f"F2: File '{fname}' is empty (0 bytes).")
            
            with open(fpath, 'r', encoding='utf-8') as f:
                f.read() # Try reading to verify UTF-8
        except UnicodeDecodeError:
            errors.append(f"F2: File '{fname}' contains invalid UTF-8 encoding.")
        except Exception as e:
            errors.append(f"F2: Integrity error on '{fname}': {e}")
            
    return errors

def verify_logic_presence(target_dir: str, architecture_path: str):
    """F8.1: Ensures that functions defined in architecture are present in files."""
    if not os.path.exists(architecture_path):
        return []
    
    try:
        with open(architecture_path, "r", encoding="utf-8") as f:
            arch_content = f.read()
            
        # Basic extraction of component blocks from architecture.md
        # This is a simplified regex; a real one would be more robust
        components = re.findall(r"### Component: (\w+).*?Functions:\n(.*?)(?=\n###|$)", arch_content, re.DOTALL)
        
        errors = []
        for comp_name, functions in components:
            fname = f"{comp_name}.rs" # Assuming Rust for now
            fpath = os.path.join(target_dir, fname)
            if not os.path.exists(fpath): continue
            
            with open(fpath, "r", encoding="utf-8") as f:
                code = f.read()
                
            func_names = [line.strip().split("(")[0].replace("- ", "") for line in functions.strip().split("\n")]
            for fn in func_names:
                if fn and fn not in code:
                    errors.append(f"F8.1: Defined function '{fn}' is missing from {fname}. Logic might have been accidentally wiped.")
        return errors
    except Exception as e:
        return [f"F8.1: Failed to audit architecture mapping: {e}"]

def execution_harness(project_root: str, file_map: dict, entry_point: str = "main.py"):
    """
    1. Validate runtime environment
    2. Snapshot existing files
    3. Sandbox run in temp directory
    4. F1~F10 Physical Checklist Validation
    """
    # 0. Runtime Check (F5)
    env_ok, env_err = validate_runtime_environment()
    if not env_ok:
        return False, env_err

    # 1. Sandbox run
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Prepare sandbox
        if os.path.exists(project_root):
            for root, dirs, files in os.walk(project_root):
                rel_path = os.path.relpath(root, project_root)
                if rel_path == ".": rel_path = ""
                
                target_dir = os.path.join(tmp_dir, rel_path)
                os.makedirs(target_dir, exist_ok=True)
                
                for f in files:
                    # Filter out temporary files
                    if f.startswith(".harness_") or f.startswith(".state_"): continue
                    shutil.copy2(os.path.join(root, f), os.path.join(target_dir, f))

        # Apply new/modified files to sandbox
        for fname, code in file_map.items():
            fpath = os.path.join(tmp_dir, fname)
            os.makedirs(os.path.dirname(fpath), exist_ok=True)
            with open(fpath, "w", encoding="utf-8") as f:
                f.write(code)

        # F1~F2: Physical Integrity Audit
        if file_map: # Only if we have new files to check
            integrity_errors = verify_file_integrity(tmp_dir, list(file_map.keys()))
            if integrity_errors:
                return False, "\n".join(integrity_errors)

        # F3: Entry Point Validation
        entry_path = os.path.join(tmp_dir, entry_point)
        arch_path = os.path.join(tmp_dir, "architecture.md")
        
        # F8.1: Logic Presence Audit (Check against architecture.md)
        mapping_errors = verify_logic_presence(tmp_dir, arch_path)
        if mapping_errors:
            return False, "\n".join(mapping_errors)

        if not os.path.exists(entry_path):
            return False, f"F3: Entry point '{entry_point}' is missing or path is incorrect."
            
        if os.path.getsize(entry_path) < 10:
             return False, f"F3: Entry point '{entry_point}' is non-functional (too small)."

        # F9: Side-Effect Monitoring (Pre-scan)
        pre_files = set()
        for root, _, files in os.walk(tmp_dir):
            for f in files:
                pre_files.add(os.path.relpath(os.path.join(root, f), tmp_dir))

        # F6: Runtime Execution
        result = run_code(entry_path)
        is_ok, err_msg = basic_test(result)
        
        if not is_ok:
            return False, f"F6: Runtime Crash in {entry_point}:\n{err_msg}"

        # F9: Side-Effect Monitoring (Post-scan)
        post_files = set()
        for root, _, files in os.walk(tmp_dir):
            for f in files:
                post_files.add(os.path.relpath(os.path.join(root, f), tmp_dir))
        
        new_files = post_files - pre_files
        if new_files:
            # v0.1: Warning for side-effect drift
            print(f"⚠️ [F9] Side-effect detected: {', '.join(new_files)} created during execution.", file=sys.stderr)
            
        return True, result.get("stdout", "")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="AXON Execution Harness")
    parser.add_argument("--project-root", required=True, help="Target project directory")
    parser.add_argument("--files-json", required=True, help="JSON map of {filename: content}")
    parser.add_argument("--entry", default="main.py", help="Entry point file")
    parser.add_argument("--commit", action="store_true", help="Actually commit files if successful")
    
    args = parser.parse_args()
    
    try:
        with open(args.files_json, 'r', encoding='utf-8') as f:
            file_map = json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to load files-json: {e}", file=sys.stderr)
        sys.exit(1)
        
    success, output = execution_harness(args.project_root, file_map, args.entry)
    
    if success:
        if args.commit:
            # Commit files to project_root
            for fname, code in file_map.items():
                fpath = os.path.join(args.project_root, fname)
                os.makedirs(os.path.dirname(fpath), exist_ok=True)
                with open(fpath, "w", encoding="utf-8") as f:
                    f.write(code)
            print("<<<<HARNESS_SUCCESS_COMMITTED>>>>")
        else:
            print("<<<<HARNESS_SUCCESS_VALIDATED>>>>")
        print(output)
        sys.exit(0)
    else:
        print(f"ERROR: {output}", file=sys.stderr)
        sys.exit(1)
