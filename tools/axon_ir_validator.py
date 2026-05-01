#!/usr/bin/env python3
# encoding: utf-8
"""
AXON IR Validator (Semantic Guardian)
Validates the resulting code state against constraints.json.
"""
import ast
import json
import sys
import os
import re

def extract_columns_from_sql(code: str):
    """Simple regex to extract columns from CREATE TABLE statements."""
    cols = []
    # Look for CREATE TABLE ... ( col1 TYPE, col2 TYPE, ... )
    # This is a heuristic but good enough for 80% coverage in Stage 6
    matches = re.findall(r'(\w+)\s+(INTEGER|TEXT|DATE|TIMESTAMP)', code, re.IGNORECASE)
    return set(matches)

def validate_imports(code: str, strategy: str, allow: list, deny: list):
    """AST-based import validation."""
    try:
        tree = ast.parse(code)
    except SyntaxError as e:
        return False, f"Syntax Error: {e}"
        
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name in deny or ("*" in deny and alias.name not in allow):
                    return False, f"Forbidden Import: 'import {alias.name}' is not allowed."
        elif isinstance(node, ast.ImportFrom):
            mod = node.module or ""
            if mod in deny or ("*" in deny and mod not in allow):
                return False, f"Forbidden Import: 'from {mod} import ...' is not allowed."
    return True, None

def validate_functions(code: str, required: list, forbid_extra: bool):
    """AST-based function signature validation."""
    try:
        tree = ast.parse(code)
    except SyntaxError as e:
        return False, f"Syntax Error: {e}"
        
    found_funcs = {}
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            found_funcs[node.name] = [arg.arg for arg in node.args.args]
            
    for req in required:
        name = req["name"]
        if name not in found_funcs:
            return False, f"Missing Required Function: '{name}' not found."
        
        # Check args if specified
        if "args" in req:
            if found_funcs[name] != req["args"]:
                return False, f"Function Signature Mismatch: '{name}' expected args {req['args']}, got {found_funcs[name]}."
                
    if forbid_extra:
        req_names = {r["name"] for r in required}
        for name in found_funcs:
            if name not in req_names:
                return False, f"Forbidden Extra Function: '{name}' is not allowed in this module."
                
    return True, None

FORBIDDEN_CALLS = {
    "open", "eval", "exec", "__import__", 
    "subprocess.run", "subprocess.Popen", "os.system", 
    "os.remove", "os.unlink", "shutil.rmtree"
}

def validate_behavior(code: str):
    """AST-based behavior validation for side effects."""
    try:
        tree = ast.parse(code)
    except SyntaxError as e:
        return False, f"Syntax Error: {e}"
        
    for node in ast.walk(tree):
        # 1. Check for forbidden function calls
        if isinstance(node, ast.Call):
            func_name = ""
            if isinstance(node.func, ast.Name):
                func_name = node.func.id
            elif isinstance(node.func, ast.Attribute):
                # Handle cases like os.system or subprocess.run
                if isinstance(node.func.value, ast.Name):
                    func_name = f"{node.func.value.id}.{node.func.attr}"
            
            if func_name in FORBIDDEN_CALLS:
                return False, f"Forbidden Call: '{func_name}' is a prohibited side-effect."
        
        # 2. Check for Global State Mutation
        if isinstance(node, ast.Global):
            return False, "Forbidden Global: Global state modification is prohibited."
            
    return True, None

def validate_file(filename: str, code: str, config: dict, before_code: str = ""):
    """Performs all checks for a single file based on its IR config."""
    mode = config.get("mode", "free")
    
    # 0. Behavior Check (Global for all files in Stage 6)
    ok, err = validate_behavior(code)
    if not ok: return False, f"Behavior Violation in {filename}: {err}"
    
    # 1. Structural Check (Frozen)
    if mode == "frozen":
        # Check if code structure changed significantly (e.g. schema)
        if "schema" in config:
            before_cols = extract_columns_from_sql(before_code)
            after_cols = extract_columns_from_sql(code)
            if before_cols != after_cols:
                return False, f"Structural Violation in {filename}: Schema modification is forbidden in frozen mode. Expected {before_cols}, got {after_cols}."

    # 2. Imports Check
    if "imports" in config:
        imp_cfg = config["imports"]
        ok, err = validate_imports(code, imp_cfg.get("strategy", ""), imp_cfg.get("allow", []), imp_cfg.get("deny", []))
        if not ok: return False, f"Import Violation in {filename}: {err}"
        
    # 3. Functions Check
    if "functions" in config:
        func_cfg = config["functions"]
        ok, err = validate_functions(code, func_cfg.get("required", []), func_cfg.get("forbid_extra_functions", False))
        if not ok: return False, f"Function Violation in {filename}: {err}"
        
    return True, None

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: axon_ir_validator.py <constraints.json> <state.json> [project_root]")
        sys.exit(1)
        
    try:
        with open(sys.argv[1], "r") as f: constraints = json.load(f)
        with open(sys.argv[2], "r") as f: state = json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to load input JSON: {e}", file=sys.stderr)
        sys.exit(1)
        
    project_root = sys.argv[3] if len(sys.argv) > 3 else "."
    
    file_configs = constraints.get("files", {})
    all_ok = True
    errors = []
    
    for filename, code in state.items():
        if filename == "error" or (isinstance(code, dict) and "error" in code):
            err_msg = code if isinstance(code, str) else code.get("error", "Unknown error")
            errors.append(f"Simulation Error in {filename}: {err_msg}")
            all_ok = False
            continue
            
        if filename in file_configs:
            # Load original code if needed for comparison
            before_code = ""
            orig_path = os.path.join(project_root, filename)
            if os.path.exists(orig_path):
                with open(orig_path, "r", encoding="utf-8") as f:
                    before_code = f.read()
            
            ok, err = validate_file(filename, code, file_configs[filename], before_code)
            if not ok:
                all_ok = False
                errors.append(err)
                
    if all_ok:
        print("<<<<VALIDATION_SUCCESS>>>>")
        sys.exit(0)
    else:
        print("<<<<VALIDATION_FAILED>>>>")
        # Output structured trace for Self-Healing
        trace = {
            "stage": "semantic_validation",
            "errors": errors
        }
        print(json.dumps(trace, indent=2))
        sys.exit(1)
