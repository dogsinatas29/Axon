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

def execution_harness(project_root: str, file_map: dict, entry_point: str = "main.py"):
    """
    1. Validate runtime environment
    2. Snapshot existing files
    3. Sandbox run in temp directory
    4. If success, commit to project_root
    """
    # 0. Runtime Check
    env_ok, env_err = validate_runtime_environment()
    if not env_ok:
        return False, env_err

    # 1. Sandbox run
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Prepare sandbox
        # Copy existing files first to simulate full project
        if os.path.exists(project_root):
            for root, dirs, files in os.walk(project_root):
                rel_path = os.path.relpath(root, project_root)
                if rel_path == ".":
                    rel_path = ""
                
                target_dir = os.path.join(tmp_dir, rel_path)
                os.makedirs(target_dir, exist_ok=True)
                
                for f in files:
                    shutil.copy2(os.path.join(root, f), os.path.join(target_dir, f))

        # Apply new/modified files to sandbox
        for fname, code in file_map.items():
            fpath = os.path.join(tmp_dir, fname)
            os.makedirs(os.path.dirname(fpath), exist_ok=True)
            with open(fpath, "w", encoding="utf-8") as f:
                f.write(code)

        # Execute
        entry_path = os.path.join(tmp_dir, entry_point)
        if not os.path.exists(entry_path):
            return False, f"CRITICAL: Entry point '{entry_point}' is missing. Every project MUST have an executable entry point."
            
        # Check if file is empty
        if os.path.getsize(entry_path) < 10:
             return False, f"CRITICAL: Entry point '{entry_point}' is empty or too small to be functional."

        result = run_code(entry_path)
        is_ok, err_msg = basic_test(result)
        
        if not is_ok:
            return False, f"Runtime Error in {entry_point}:\n{err_msg}"
            
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
