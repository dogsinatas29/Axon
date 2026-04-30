#!/usr/bin/env python3
# encoding: utf-8
import ast
import json
import re
import sys
import os
from pathlib import Path

# 1. Code Indexer (AST based)
class CodeIndexer:
    @staticmethod
    def index_file(path):
        try:
            tree = ast.parse(Path(path).read_text(encoding='utf-8'))
            funcs, classes = [], []
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    funcs.append(node.name)
                if isinstance(node, ast.ClassDef):
                    classes.append(node.name)
            return {
                "functions": set(funcs),
                "classes": set(classes),
                "content": code
            }
        except Exception as e:
            return {"error": str(e)}

    @staticmethod
    def build_index(root_path):
        idx = {}
        for p in Path(root_path).rglob("*.py"):
            # Skip hidden files and virtual envs
            if any(part.startswith(".") for part in p.parts):
                continue
            idx[str(p.relative_to(root_path))] = CodeIndexer.index_file(p)
        return idx

# 2. Architecture Parser
class ArchParser:
    SPEC_RE = re.compile(r'<!-- AXON:SPEC:COMPONENTS(.*?)-->', re.S)

    @staticmethod
    def parse_arch(arch_path):
        content = Path(arch_path).read_text(encoding='utf-8')
        m = ArchParser.SPEC_RE.search(content)
        if not m:
            raise ValueError("Missing AXON:SPEC:COMPONENTS block in architecture.md")
        return json.loads(m.group(1).strip())

# 3. Main Mapping Validator
def validate_mapping(arch_path, project_root, state_json_path=None):
    try:
        arch = ArchParser.parse_arch(arch_path)
        
        if state_json_path:
            with open(state_json_path, 'r', encoding='utf-8') as f:
                state = json.load(f)
            # Normalize state to match build_index output format
            code_idx = {}
            for fname, code in state.items():
                try:
                    tree = ast.parse(code)
                    funcs, classes = [], []
                    for node in ast.walk(tree):
                        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                            funcs.append(node.name)
                        if isinstance(node, ast.ClassDef):
                            classes.append(node.name)
                    code_idx[fname] = {
                        "functions": set(funcs),
                        "classes": set(classes),
                        "content": code
                    }
                except:
                    code_idx[fname] = {"error": "AST parse failed"}
        else:
            code_idx = CodeIndexer.build_index(project_root)
            
        # 4. Normalize paths for matching
        normalized_idx = {os.path.normpath(k): v for k, v in code_idx.items()}
        
        errors = []
        for comp in arch.get("components", []):
            fname = comp.get("file")
            if not fname: continue
            
            norm_fname = os.path.normpath(fname)

            # File Check
            if norm_fname not in normalized_idx:
                found_files = ", ".join(list(normalized_idx.keys()))
                errors.append(f"[MISSING_FILE] {fname} (Normalized: {norm_fname}). Found in index: [{found_files}]")
                continue

            # Symbol Check
            symbols = comp.get("symbols", [])
            file_meta = normalized_idx[norm_fname]
            code_content = file_meta.get("content", "")
            
            # Skip symbol check if it's still a stub
            if "AXON STUB" in code_content:
                continue

            available = file_meta.get("functions", set()) | file_meta.get("classes", set())

            for s in symbols:
                if s not in available:
                    errors.append(f"[MISSING_SYMBOL] {fname}:{s}")

            # Type Enforcement (Entry check)
            ctype = comp.get("type")
            if ctype == "entry":
                # Check if at least one of the intended symbols exists
                if not any(s in available for s in symbols):
                    errors.append(f"[INVALID_ENTRY] {fname} is marked as 'entry' but missing defined entry functions: {', '.join(symbols)}")

        return errors
    except Exception as e:
        return [f"[VALIDATOR_ERROR] {str(e)}"]

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("arch_path")
    parser.add_argument("project_root")
    parser.add_argument("--state-json", help="Path to simulated state JSON")
    args = parser.parse_args()
    
    errors = validate_mapping(args.arch_path, args.project_root, args.state_json)
    
    if not errors:
        print("<<<<MAPPING_VALIDATION_SUCCESS>>>>")
        sys.exit(0)
    else:
        print("<<<<MAPPING_VALIDATION_FAILED>>>>")
        for err in errors:
            print(err)
        sys.exit(1)
