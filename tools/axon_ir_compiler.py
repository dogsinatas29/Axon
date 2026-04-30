#!/usr/bin/env python3
# encoding: utf-8
"""
AXON IR Compiler
Extracts machine-readable spec blocks from architecture.md and generates constraints.json.
"""
import re
import json
import sys
import os
import argparse

SPEC_PATTERN = re.compile(
    r"```spec:(?P<type>\w+)(?:\s+)\n?(?P<body>.*?)```",
    re.DOTALL
)
COMMENT_PATTERN = re.compile(
    r"<!-- AXON:SPEC:(?P<type>\w+)(?:\s+)\n?(?P<body>.*?)-->",
    re.DOTALL
)

def extract_spec_blocks(md: str):
    blocks = []
    # Try triple backtick format
    for match in SPEC_PATTERN.finditer(md):
        try:
            blocks.append({
                "type": match.group("type").lower(),
                "data": json.loads(match.group("body").strip())
            })
        except json.JSONDecodeError as e:
            print(f"ERROR: Invalid JSON in spec:{match.group('type')} block: {e}", file=sys.stderr)
            sys.exit(1)
            
    # Try HTML comment format (used by some models)
    for match in COMMENT_PATTERN.finditer(md):
        try:
            blocks.append({
                "type": match.group("type").lower(),
                "data": json.loads(match.group("body").strip())
            })
        except json.JSONDecodeError as e:
            print(f"ERROR: Invalid JSON in <!-- AXON:SPEC:{match.group('type')} --> block: {e}", file=sys.stderr)
            sys.exit(1)
            
    return blocks

def validate_blocks(blocks):
    if not blocks:
        print(f"ERROR: No spec blocks found in {args.input}. IR Compilation aborted.", file=sys.stderr)
        sys.exit(1)
        
    seen_db = False
    for b in blocks:
        if b["type"] == "db":
            if seen_db:
                print("ERROR: Duplicate spec:db block found.", file=sys.stderr)
                sys.exit(1)
            seen_db = True
            # Basic validation of DB schema structure
            data = b["data"]
            if "table" not in data or "columns" not in data:
                print("ERROR: spec:db block missing 'table' or 'columns' field.", file=sys.stderr)
                sys.exit(1)
        elif b["type"] == "function":
            # Basic validation of function contract
            data = b["data"]
            if "name" not in data or "args" not in data:
                print("ERROR: spec:function block missing 'name' or 'args' field.", file=sys.stderr)
                sys.exit(1)

def compile_constraints(blocks):
    constraints = {
        "version": "0.0.19-hardened",
        "global": {
            "output_format": "json_ast",
            "allow_file_create": False,
            "allow_file_delete": False
        },
        "files": {}
    }
    
    for block in blocks:
        b_type = block["type"]
        data = block["data"]
        
        if b_type == "db":
            # Map to database.py by default or use a specific field if added
            filename = data.get("file", "database.py")
            constraints["files"][filename] = {
                "mode": "frozen",
                "schema": data,
                "constraints": {
                    "deny": ["add_column", "remove_column", "rename_column", "change_type", "add_constraint"]
                }
            }
        elif b_type == "function":
            filename = data.get("file", "calculator.py")
            if filename not in constraints["files"]:
                constraints["files"][filename] = {
                    "mode": "restricted",
                    "functions": {"required": []},
                    "imports": {"strategy": "whitelist_strict", "allow": [], "deny": ["*"]}
                }
            
            constraints["files"][filename]["functions"]["required"].append({
                "name": data["name"],
                "args": data["args"],
                "returns": data.get("returns"),
                "no_extra_args": True
            })
            
            if "imports" in data:
                constraints["files"][filename]["imports"]["allow"].extend(data["imports"])
                
    return constraints

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="AXON IR Compiler")
    parser.add_argument("input", help="Path to architecture.md")
    parser.add_argument("--output", default="constraints.json", help="Path to output constraints.json")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.input):
        print(f"ERROR: {args.input} not found.", file=sys.stderr)
        sys.exit(1)
        
    with open(args.input, "r", encoding="utf-8") as f:
        md_content = f.read()
        
    blocks = extract_spec_blocks(md_content)
    validate_blocks(blocks)
    
    constraints = compile_constraints(blocks)
    
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(constraints, f, indent=2)
        
    print(f"SUCCESS: Compiled {len(blocks)} spec blocks into {args.output}")
