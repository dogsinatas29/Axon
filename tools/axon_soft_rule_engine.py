#!/usr/bin/env python3
# encoding: utf-8
import json
import sys
import os
import ast
from pathlib import Path
from collections import defaultdict, deque

# Stage 4.7 & 4.8.8: Advanced AST-based Tools
def has_import(code, target_module):
    try:
        tree = ast.parse(code)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for n in node.names:
                    if n.name == target_module or n.name.startswith(target_module + "."):
                        return True
            if isinstance(node, ast.ImportFrom):
                if node.module == target_module or (node.module and node.module.startswith(target_module + ".")):
                    return True
        return False
    except:
        return False

def extract_deps(code):
    deps = set()
    try:
        tree = ast.parse(code)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for n in node.names:
                    deps.add(n.name.split('.')[0])
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    deps.add(node.module.split('.')[0])
    except:
        pass
    return deps

def get_affected_files(changed_files, state):
    # 1. Build Dependency Graphs
    forward_graph = defaultdict(set)
    reverse_graph = defaultdict(set)
    
    # Map module names to potential filenames (conservative)
    module_to_file = {}
    for fname in state.keys():
        mname = os.path.splitext(fname)[0]
        module_to_file[mname] = fname

    for fname, code in state.items():
        deps = extract_deps(code)
        for d in deps:
            if d in module_to_file:
                target_file = module_to_file[d]
                forward_graph[fname].add(target_file)
                reverse_graph[target_file].add(fname)

    # 2. Propagate through both graphs
    affected = set(changed_files)
    queue = deque(changed_files)
    
    while queue:
        current = queue.popleft()
        # Both forward (what I use) and reverse (who uses me)
        for nxt in forward_graph[current] | reverse_graph[current]:
            if nxt not in affected:
                affected.add(nxt)
                queue.append(nxt)
    
    return affected

from axon_profile_resolver import ProfileResolver

def check_soft_rules(state_json_path, suggestions_path, changed_files=None, profiles_db_path=".axon_trace/profiles.json"):
    if not os.path.exists(suggestions_path):
        return []

    try:
        with open(suggestions_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            suggestions = data.get("suggested_rules", [])
            # Also support profiles within the same file if present
            profiles_db = data.get("profiles", {})
            
        if os.path.exists(profiles_db_path):
            with open(profiles_db_path, 'r', encoding='utf-8') as f:
                profiles_db.update(json.load(f))

        with open(state_json_path, 'r', encoding='utf-8') as f:
            state = json.load(f)

        resolver = ProfileResolver(profiles_db)

        # Stage 4.8.8: Dependency-Aware Filtering
        if changed_files:
            affected_set = get_affected_files(changed_files, state)
            suggestions = [s for s in suggestions if s.get("file") in affected_set or not s.get("file")]

        # Build index
        code_idx = {}
        for fname, code in state.items():
            # ... (parsing remains same)
            try:
                tree = ast.parse(code)
                funcs = [node.name for node in ast.walk(tree) if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))]
                classes = [node.name for node in ast.walk(tree) if isinstance(node, ast.ClassDef)]
                code_idx[fname] = {"functions": set(funcs), "classes": set(classes), "code": code}
            except:
                code_idx[fname] = {"error": "AST_FAIL"}

        violations = []
        for fname, idx in code_idx.items():
            if "error" in idx: continue
            
            # 1. Resolve Rules for this specific file
            # (In a real system, file-to-profile mapping would be in constraints.json)
            # For now, we simulate this by finding rules targeting this file
            file_profiles = [] # Placeholder for profile mapping
            file_overrides = {} # Placeholder for overrides
            
            resolved_rules, trace = resolver.resolve(file_profiles, file_overrides)
            
            # 2. Merge with individual suggested rules for this file
            file_rules = [s for s in suggestions if s.get("file") == fname]
            
            # 3. Perform Validation (Unified)
            for rule in file_rules:
                rtype = rule.get("type")
                if rtype == "require_symbol":
                    symbol = rule.get("symbol")
                    available = idx["functions"] | idx["classes"]
                    if symbol not in available:
                        violations.append({"rule": rule, "error": "MISSING_SYMBOL", "trace": trace.get(f"symbol:{symbol}")})
                
                if rtype == "require_import":
                    module = rule.get("module")
                    if not has_import(idx["code"], module):
                        violations.append({"rule": rule, "error": "MISSING_IMPORT", "trace": trace.get(f"import:{module}")})

        return violations
    except Exception as e:
        return [{"error": "ENGINE_ERROR", "detail": str(e)}]

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: axon_soft_rule_engine.py <state_json> <suggestions_json> [changed_file1,changed_file2]")
        sys.exit(0)

    changed = None
    if len(sys.argv) > 3:
        changed = sys.argv[3].split(',')

    violations = check_soft_rules(sys.argv[1], sys.argv[2], changed)
    
    if not violations:
        print("<<<<SOFT_RULES_PASSED>>>>")
        sys.exit(0)
    else:
        print("<<<<SOFT_RULES_VIOLATION>>>>")
        for v in violations:
            print(json.dumps(v))
        sys.exit(0)
