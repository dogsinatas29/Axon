#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Golden Tester (Business Logic Guardian)
Validates the simulated code state against functional invariants.
"""
import json
import sys
import os
import importlib.util
import tempfile
import shutil

def run_functional_test(filename, code, tests, project_root):
    """Executes a functional test against a specific file's code."""
    results = []
    
    # Create a temporary module to load the code
    with tempfile.NamedTemporaryFile(suffix=".py", mode='w', encoding='utf-8', delete=False) as tmp:
        tmp.write(code)
        tmp_path = tmp.name

    try:
        spec = importlib.util.spec_from_file_location("virtual_mod", tmp_path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        
        for test in tests:
            func_name = test.get("function")
            inputs = test.get("input", [])
            expected = test.get("expect")
            
            if not hasattr(module, func_name):
                results.append({"status": "fail", "detail": f"Function '{func_name}' not found in {filename}"})
                continue
            
            func = getattr(module, func_name)
            try:
                # Support single or multiple inputs
                if isinstance(inputs, list):
                    actual = func(*inputs)
                else:
                    actual = func(inputs)
                
                # Simple evaluation of the 'expect' string or value
                # In a real system, this would use a more robust matcher
                is_pass = False
                if isinstance(expected, str) and (expected.startswith(">") or expected.startswith("<") or expected.startswith("==")):
                    is_pass = eval(f"{actual} {expected}")
                else:
                    is_pass = (actual == expected)
                
                if is_pass:
                    results.append({"status": "pass", "input": inputs, "actual": actual})
                else:
                    results.append({"status": "fail", "input": inputs, "expect": expected, "actual": actual})
            except Exception as e:
                results.append({"status": "error", "input": inputs, "detail": str(e)})
                
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)
            
    return results

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: axon_golden_tester.py <constraints.json> <state.json>")
        sys.exit(1)
        
    try:
        with open(sys.argv[1], "r") as f: constraints = json.load(f)
        with open(sys.argv[2], "r") as f: state = json.load(f)
    except Exception as e:
        print(json.dumps({"stage": "golden_test", "status": "error", "detail": str(e)}))
        sys.exit(1)
        
    test_configs = constraints.get("tests", {})
    all_results = {}
    total_fail = 0
    
    for filename, tests in test_configs.items():
        if filename in state:
            code = state[filename]
            res = run_functional_test(filename, code, tests, ".")
            all_results[filename] = res
            total_fail += sum(1 for r in res if r["status"] != "pass")
            
    if total_fail == 0:
        print("<<<<GOLDEN_TEST_SUCCESS>>>>")
        print(json.dumps(all_results, indent=2))
        sys.exit(0)
    else:
        print("<<<<GOLDEN_TEST_FAILED>>>>")
        print(json.dumps(all_results, indent=2))
        sys.exit(1)
