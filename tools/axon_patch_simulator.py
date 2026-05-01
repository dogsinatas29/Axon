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
    results = {}
    
    # 1. Load all existing files from project_root into the virtual state FIRST
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

    # 2. Extract JSON from potentially conversational response
    raw_json = junior_output_json
    if "```json" in junior_output_json:
        try:
            raw_json = junior_output_json.split("```json")[1].split("```")[0].strip()
        except:
            pass
    elif "[" in junior_output_json:
        try:
            start = junior_output_json.find("[")
            end = junior_output_json.rfind("]")
            if start != -1 and end != -1:
                raw_json = junior_output_json[start:end+1]
        except:
            pass

    try:
        data = json.loads(raw_json)
    except Exception as e:
        # v0.0.22: Hard Fail on parse error
        print(f"ERROR: JSON Parse Error: {e}. Raw: {raw_json[:100]}...", file=sys.stderr)
        sys.exit(1)
        
    if not isinstance(data, list):
        data = [data] # Handle single object or list
        
    # 3. Apply patches from Junior agent
    for item in data:
        target = item.get("target")
        if not target: continue
        
        # Normalize target path to match our results keys
        target = os.path.normpath(target)
        
        base_code = results.get(target, "")
        op_type = item.get("type", "rewrite")
        
        if op_type == "rewrite":
            new_code = item.get("code", "")
            # v0.0.23: Stub Detection - Eradicate "AXON STUB"
            if "AXON STUB" in new_code:
                results[f"error_{target}"] = f"ERROR: Proposed code for {target} still contains AXON STUB marker. You must completely replace the stub with actual implementation."
            else:
                results[target] = new_code
        elif op_type == "patch":
            diff = item.get("diff", "")
            new_code = apply_diff(base_code, diff)
            if new_code.startswith("ERROR:"):
                results[f"error_{target}"] = new_code
            else:
                # v0.0.23: Stub Detection - Eradicate "AXON STUB" in patched results
                if "AXON STUB" in new_code:
                    results[f"error_{target}"] = f"ERROR: After patch, {target} still contains AXON STUB marker. Patches must remove the stub prefix."
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
