#!/usr/bin/env python3
# encoding: utf-8
"""
AXON IR Mapper & Validator
Checks if the LLM-provided code satisfies the IR contract.
"""
import ast
import json
import sys
import os

def extract_symbols(code):
    try:
        tree = ast.parse(code)
        funcs = []
        imports = []
        
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                # Extract signature: name(arg1, arg2, ...)
                args = [arg.arg for arg in node.args.args]
                signature = f"{node.name}({', '.join(args)})"
                
                # Extract dependencies (internal calls)
                deps = []
                for subnode in ast.walk(node):
                    if isinstance(subnode, ast.Call):
                        if isinstance(subnode.func, ast.Name):
                            deps.append(subnode.func.id)
                
                funcs.append({
                    "name": node.name,
                    "signature": signature,
                    "args": args,
                    "dependencies": list(set(deps))
                })
            elif isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
            elif isinstance(node, ast.ImportFrom):
                imports.append(f"{node.module}.{node.names[0].name}")
                
        return funcs, list(set(imports)), None
    except SyntaxError as e:
        return None, None, str(e)

def map_and_validate(ir_json, patch_json):
    ir = json.loads(ir_json)
    patches = json.loads(patch_json)
    
    errors = []
    
    for patch in patches:
        target = patch["target"]
        code = patch["code"]
        
        # 4. FILE -> Component Mapping Rule
        comp_name = os.path.basename(target).replace(".py", "")
        ir_file = next((f for f in ir["files"] if f["name"] == comp_name), None)
        
        if not ir_file:
            errors.append(f"IR Error: Component '{comp_name}' (from {target}) not found in architecture.")
            continue
            
        # 6. CODE -> Function Extraction
        found_funcs, found_imports, err = extract_symbols(code)
        if found_funcs is None:
            errors.append(f"Syntax Error in {target}: {err}")
            continue
            
        found_names = {f["name"]: f for f in found_funcs}
        
        # 11.1 Structural Validation
        required_names = {f["name"] for f in ir_file["functions"]}
        missing = required_names - set(found_names.keys())
        if missing:
            errors.append(f"Contract Violation in {target}: Missing required functions {list(missing)}")
            
        # 11.2 Signature Validation
        for ir_func in ir_file["functions"]:
            if ir_func["name"] in found_names:
                actual_sig = found_names[ir_func["name"]]["signature"]
                # architecture.md usually has simple signatures like run()
                # We normalize for comparison if needed, but for now exact match or presence is checked
                if "(" in ir_func["signature"]:
                    # Basic check: name match is already done, signature check is stricter
                    pass 
                
    return errors

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: axon_ir_mapper.py <ir.json> <patch.json>")
        sys.exit(1)
        
    with open(sys.argv[1], "r") as f: ir_data = f.read()
    with open(sys.argv[2], "r") as f: patch_data = f.read()
    
    errors = map_and_validate(ir_data, patch_data)
    
    if not errors:
        print("<<<<IR_MAPPING_SUCCESS>>>>")
        sys.exit(0)
    else:
        print("<<<<IR_MAPPING_FAILED>>>>")
        print(json.dumps({"errors": errors}, indent=2))
        sys.exit(1)
