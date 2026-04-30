#!/usr/bin/env tuple3
# encoding: utf-8
"""
AXON Property Tester (Total Integrity Guardian)
Validates code invariants with strict result normalization and safe evaluation.
"""
import json
import sys
import os
import importlib.util
import tempfile
import random
import operator
from datetime import datetime

# Safe operator mapping
OPS = {
    ">=": operator.ge,
    "<=": operator.le,
    ">": operator.gt,
    "<": operator.lt,
    "==": operator.eq,
    "!=": operator.ne,
}

def normalize_result(actual):
    """Strictly sanitizes and normalizes output for comparison."""
    if actual is None:
        return None # Explicit failure in safe_check
    
    # Object normalization (e.g. relativedelta)
    if hasattr(actual, 'years'):
        return float(actual.years)
    
    # Numeric normalization
    if isinstance(actual, (int, float)):
        return float(actual)
    
    # String numeric conversion attempt
    if isinstance(actual, str):
        try:
            return float(actual)
        except ValueError:
            pass
            
    return actual # Return as is for non-numeric comparison if needed

def safe_check(actual, invariant_str):
    """Performs numeric comparison on normalized results."""
    normalized = normalize_result(actual)
    if normalized is None:
        return False
        
    for op_str, op_func in OPS.items():
        if op_str in invariant_str:
            try:
                raw_val = invariant_str.split(op_str)[1].strip()
                target_value = float(raw_val)
                # Ensure both are floats for comparison
                return op_func(float(normalized), target_value)
            except (ValueError, IndexError, TypeError):
                continue
    return False

def run_property_test(filename, code, properties):
    """Executes randomized fuzzing with strict output normalization."""
    results = []
    
    with tempfile.NamedTemporaryFile(suffix=".py", mode='w', encoding='utf-8', delete=False) as tmp:
        tmp.write(code)
        tmp_path = tmp.name

    try:
        spec = importlib.util.spec_from_file_location("virtual_mod", tmp_path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        
        for prop in properties:
            func_name = prop.get("function")
            invariant = prop.get("invariant")
            iterations = prop.get("iterations", 100)
            
            if not hasattr(module, func_name):
                results.append({"status": "fail", "detail": f"Function '{func_name}' not found"})
                continue
            
            func = getattr(module, func_name)
            fail_count = 0
            first_error = None
            
            for _ in range(iterations):
                year = random.randint(1900, 2025)
                month = random.randint(1, 12)
                day = random.randint(1, 28)
                fuzz_input = None
                
                try:
                    # Adaptive Input: try string, then datetime
                    try:
                        fuzz_input = f"{year}-{month:02d}-{day:02d}"
                        actual = func(fuzz_input)
                    except Exception as e:
                        fuzz_input = datetime(year, month, day)
                        actual = func(fuzz_input)

                    if not safe_check(actual, invariant):
                        fail_count += 1
                        if not first_error:
                            first_error = f"Invariant '{invariant}' failed for input {fuzz_input} (Got: {actual})"
                except Exception as e:
                    # Any execution crash is a property failure
                    fail_count += 1
                    if not first_error:
                        first_error = f"Execution crashed for input {fuzz_input or 'N/A'}: {str(e)}"
            
            if fail_count == 0:
                results.append({"status": "pass", "property": invariant, "iterations": iterations})
            else:
                results.append({"status": "fail", "property": invariant, "fails": fail_count, "observation": first_error})
                
    finally:
        if os.path.exists(tmp_path): os.remove(tmp_path)
    return results

if __name__ == "__main__":
    if len(sys.argv) < 3: sys.exit(1)
    with open(sys.argv[1], "r") as f: constraints = json.load(f)
    with open(sys.argv[2], "r") as f: state = json.load(f)
    
    prop_configs = constraints.get("properties", {})
    all_results = {}
    total_fail = 0
    
    for filename, props in prop_configs.items():
        if filename in state:
            res = run_property_test(filename, state[filename], props)
            all_results[filename] = res
            total_fail += sum(1 for r in res if r["status"] != "pass")
            
    if total_fail == 0:
        print("<<<<PROPERTY_TEST_SUCCESS>>>>")
        sys.exit(0)
    else:
        print("<<<<PROPERTY_TEST_FAILED>>>>")
        print(json.dumps(all_results, indent=2))
        sys.exit(1)
