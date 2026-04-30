#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Patch Simulator (Virtual FS)
Applies JSON AST patches to base files and returns the resulting code state.
"""
import sys
import json
import os
import difflib

def apply_diff(base_code: str, diff_str: str) -> str:
    """Applies a standard unified diff to base_code."""
    base_lines = base_code.splitlines(keepends=True)
    diff_lines = diff_str.splitlines(keepends=True)
    
    # We use a helper because applying raw unified diffs in python is tricky
    # Standard way: patch command, but we want zero-dependency virtual FS
    # For now, we'll implement a simple line-based patcher
    
    # Simple heuristic patcher for unified diffs
    result = []
    i = 0 # base lines ptr
    
    # Extract just the added/removed lines from diff
    # This is a VERY simplified patcher, in production we should use a proper lib
    # But for AXON v0.0.19, we'll keep it deterministic
    
    # Note: If the diff is empty or malformed, return base
    if not diff_str.strip():
        return base_code

    # Fallback to simple replacement if diff is actually just the full code
    if not diff_str.startswith("---") and not diff_str.startswith("@@"):
        return diff_str

    try:
        # We use difflib to attempt application or just use it to validate
        # Actually, let's use a more robust approach:
        import subprocess
        import tempfile
        
        with tempfile.NamedTemporaryFile(mode='w', delete=False) as bf:
            bf.write(base_code)
            bf_name = bf.name
        with tempfile.NamedTemporaryFile(mode='w', delete=False) as df:
            df.write(diff_str)
            df_name = df.name
            
        try:
            # 1. Check if patch is valid (dry run)
            check_res = subprocess.run(["patch", "--check", "-u", "-s", bf_name, df_name], capture_output=True, text=True)
            if check_res.returncode != 0:
                return f"ERROR: Patch validation failed (hunk mismatch or context error): {check_res.stderr}"

            # 2. Apply patch
            res = subprocess.run(["patch", "-u", "-s", bf_name, df_name], capture_output=True, text=True)
            if res.returncode == 0:
                with open(bf_name, "r") as f:
                    new_code = f.read()
                return new_code
            else:
                return f"ERROR: Patch application failed: {res.stderr}"
        finally:
            if os.path.exists(bf_name): os.remove(bf_name)
            if os.path.exists(df_name): os.remove(df_name)
    except:
        return "ERROR: Patch utility not found or execution failed"

def simulate_state(project_root: str, junior_output_json: str):
    """
    Parses Junior's JSON output and applies changes to a virtual copy of files.
    Returns: { "filename": "resulting_code" }
    """
    try:
        data = json.loads(junior_output_json)
    except Exception as e:
        return {"error": f"JSON Parse Error: {e}"}
        
    if not isinstance(data, list):
        data = [data] # Handle single object or list
        
    results = {}
    # 1. Load all existing files from project_root into the virtual state
    if os.path.exists(project_root):
        for root, dirs, files in os.walk(project_root):
            for f in files:
                if f.startswith('.') or f.endswith('.pyc'): continue
                rel_path = os.path.relpath(os.path.join(root, f), project_root)
                try:
                    with open(os.path.join(root, f), 'r', encoding='utf-8') as file:
                        results[rel_path] = file.read()
                except:
                    pass

    # 2. Apply patches from Junior agent
    for item in data:
        target = item.get("target")
        if not target: continue
        
        base_path = os.path.join(project_root, target)
        base_code = ""
        if os.path.exists(base_path):
            with open(base_path, "r", encoding="utf-8") as f:
                base_code = f.read()
        
        op_type = item.get("type", "rewrite")
        
        if op_type == "rewrite":
            results[target] = item.get("code", "")
        elif op_type == "patch":
            diff = item.get("diff", "")
            new_code = apply_diff(base_code, diff)
            if new_code.startswith("ERROR:"):
                results[target] = {"error": new_code}
            else:
                results[target] = new_code
                
    return results

if __name__ == "__main__":
    # Internal test or standalone usage
    if len(sys.argv) < 3:
        print("Usage: axon_patch_simulator.py <project_root> <junior_json_path>")
        sys.exit(1)
        
    with open(sys.argv[2], "r") as f:
        output = simulate_state(sys.argv[1], f.read())
    print(json.dumps(output, indent=2))
