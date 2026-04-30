#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Semantic Merge & Validator
Implements Span Patch + 3-Way Merge Engine
"""
import ast
import tokenize
from io import StringIO
import sys
import argparse
import os

def get_source_lines(code: str):
    return code.splitlines(keepends=True)

def _infer_end_lineno(code: str, start_line: int):
    lines = code.splitlines(keepends=True)
    if start_line > len(lines):
        return len(lines)
    
    src = "".join(lines[start_line-1:])
    g = tokenize.generate_tokens(StringIO(src).readline)

    indent = 0
    seen_block = False

    try:
        for tok_type, tok_str, (srow, _), (erow, _), _ in g:
            if tok_type == tokenize.INDENT:
                indent += 1
                seen_block = True
            elif tok_type == tokenize.DEDENT:
                indent -= 1
                if seen_block and indent == 0:
                    return start_line + erow - 1
    except tokenize.TokenError:
        pass # End of file or mismatched token

    return len(lines)

def extract_spans(code: str):
    if not code.strip():
        return {}
        
    try:
        tree = ast.parse(code)
    except SyntaxError:
        return {}
        
    spans = {}

    for node in ast.walk(tree):
        for child in ast.iter_child_nodes(node):
            child._parent = node

    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            parent = getattr(node, '_parent', None)
            if isinstance(parent, ast.ClassDef):
                name = f"{parent.name}.{node.name}"
                type_name = "method"
            else:
                name = node.name
                type_name = "function"
                
            start = node.lineno
            end = getattr(node, "end_lineno", None)
            if end is None:
                end = _infer_end_lineno(code, start)
            spans[(type_name, name)] = (start, end)

        if isinstance(node, ast.ClassDef):
            name = node.name
            start = node.lineno
            end = getattr(node, "end_lineno", None)
            if end is None:
                end = _infer_end_lineno(code, start)
            spans[("class", name)] = (start, end)

    return spans

def classify_changes(base_spans, local_spans, new_spans):
    keys = set(base_spans) | set(local_spans) | set(new_spans)

    result = {
        "unchanged": set(),
        "new_only": set(),
        "local_only": set(),
        "both_changed": set(),
        "only_in_new": set(),
        "only_in_local": set(),
    }

    for k in keys:
        b = k in base_spans
        l = k in local_spans
        n = k in new_spans

        if b and l and n:
            result["both_changed"].add(k)
        elif not b and n:
            result["only_in_new"].add(k)
        elif not b and l:
            result["only_in_local"].add(k)
        elif b and n and not l:
            result["new_only"].add(k)
        elif b and l and not n:
            result["local_only"].add(k)
        elif not b and not l and n:
            result["only_in_new"].add(k) # Fallback

    return result

def slice_code(lines, span):
    s, e = span
    return "".join(lines[s-1:e])

def build_patch(base_code, local_code, new_code):
    base_lines = get_source_lines(base_code)
    local_lines = get_source_lines(local_code)
    new_lines = get_source_lines(new_code)

    b = extract_spans(base_code)
    l = extract_spans(local_code)
    n = extract_spans(new_code)

    classes = classify_changes(b, l, n)
    patches = []

    for k in classes["only_in_new"] | classes["new_only"]:
        span = n[k]
        code = slice_code(new_lines, span)
        patches.append(("append", None, code))

    for k in classes["both_changed"]:
        if k[0] == "class":
            continue # Do not replace full class, rely on method spans
            
        if k in b and k in l and k in n:
            base_seg = slice_code(base_lines, b[k])
            local_seg = slice_code(local_lines, l[k])
            new_seg = slice_code(new_lines, n[k])

            if local_seg == base_seg and new_seg != base_seg:
                # Only NEW changed it
                patches.append(("replace", l[k], new_seg))
            elif local_seg != base_seg and new_seg != base_seg:
                # Both changed it!
                patches.append(("conflict", (b[k], l[k], n[k]), (base_seg, local_seg, new_seg)))
                
    # If the span didn't exist in base (2-way diff mode, base == local)
    if not b: 
        for k in l:
            if k[0] == "class":
                continue
            if k in n:
                local_seg = slice_code(local_lines, l[k])
                new_seg = slice_code(new_lines, n[k])
                if local_seg != new_seg:
                    patches.append(("replace", l[k], new_seg))

    return patches

def resolve_conflict(base_seg, local_seg, new_seg):
    return f"\n<<<<<<< LOCAL\n{local_seg}=======\n{new_seg}>>>>>>> NEW\n"

def apply_patches(local_code, patches):
    lines = get_source_lines(local_code)
    has_conflict = False

    # Sort patches by start line in reverse to avoid shifting indices when replacing
    replace_patches = [p for p in patches if p[0] in ("replace", "conflict")]
    replace_patches.sort(key=lambda p: p[1][0] if p[0] == "replace" else p[1][1][0], reverse=True)

    for op, span, data in replace_patches:
        if op == "replace":
            s, e = span
            replacement = get_source_lines(data)
            lines[s-1:e] = replacement
        elif op == "conflict":
            has_conflict = True
            b, l, n = span
            base_seg, local_seg, new_seg = data
            merged = resolve_conflict(base_seg, local_seg, new_seg)
            s, e = l
            lines[s-1:e] = get_source_lines(merged)

    # Appends go at the end
    append_patches = [p for p in patches if p[0] == "append"]
    for op, span, data in append_patches:
        if not lines:
            lines.append(data)
        else:
            if not lines[-1].endswith('\n'):
                lines[-1] += '\n'
            lines.append('\n' + data)

    return "".join(lines), has_conflict

def validate_dependencies(tree, project_root):
    if not project_root or not os.path.isdir(project_root):
        return True, None

    for node in tree.body:
        if isinstance(node, ast.ImportFrom):
            if node.level and node.level > 0:
                continue 
            module_name = node.module
            if module_name:
                path_parts = module_name.split('.')
                first_part_py = os.path.join(project_root, f"{path_parts[0]}.py")
                first_part_dir = os.path.join(project_root, path_parts[0])
                
                if os.path.exists(first_part_py) or os.path.isdir(first_part_dir):
                    full_path_py = os.path.join(project_root, *path_parts) + ".py"
                    full_path_dir = os.path.join(project_root, *path_parts, "__init__.py")
                    
                    if not (os.path.exists(full_path_py) or os.path.exists(full_path_dir)):
                        return False, f"Dependency Validation Failed: Local module '{module_name}' is imported but not found in {project_root}."
    return True, None

def merge_imports(local_code: str, new_code: str) -> str:
    local_tree = ast.parse(local_code)
    new_tree = ast.parse(new_code)
    imports_map = {}
    
    def process_tree(tree):
        for node in tree.body:
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports_map.setdefault('', set()).add((alias.name, alias.asname))
            elif isinstance(node, ast.ImportFrom):
                mod = node.module or ''
                prefix = '.' * node.level if node.level else ''
                mod_full = prefix + mod
                for alias in node.names:
                    imports_map.setdefault(mod_full, set()).add((alias.name, alias.asname))
                    
    process_tree(local_tree)
    process_tree(new_tree)
    
    lines = []
    for mod, names in sorted(imports_map.items()):
        if mod == '':
            for name, asname in sorted(names):
                if asname:
                    lines.append(f"import {name} as {asname}")
                else:
                    lines.append(f"import {name}")
        else:
            names_str = ", ".join(f"{n} as {a}" if a else n for n, a in sorted(names))
            lines.append(f"from {mod} import {names_str}")
            
    # Now remove old imports from local_code (rough approximation by filtering out lines)
    # A cleaner way is to keep local code, but we just return the block of imports to prepend
    # Actually, proper AST import merge would replace the import AST nodes.
    # We will just prepend the merged imports and let the user/formatter clean up duplicates.
    # Since it's Python, duplicated imports at top and bottom are harmless, but prepending is safe.
    import_block = "\n".join(lines)
    return import_block

def semantic_merge(target_path, new_code, base_code=None, project_root=None):
    try:
        new_tree = ast.parse(new_code)
    except SyntaxError as e:
        return False, f"Syntax Error in new code: {e.msg} at line {e.lineno}"

    if not os.path.exists(target_path):
        is_deps_valid, deps_err = validate_dependencies(new_tree, project_root)
        if not is_deps_valid:
            return False, deps_err
        return True, new_code

    with open(target_path, 'r', encoding='utf-8') as f:
        local_code = f.read()

    try:
        ast.parse(local_code)
    except SyntaxError as e:
        return False, f"Target file syntax is broken: {e.msg} at line {e.lineno}"

    is_deps_valid, deps_err = validate_dependencies(new_tree, project_root)
    if not is_deps_valid:
        return False, deps_err

    if base_code is None:
        base_code = local_code 

    patches = build_patch(base_code, local_code, new_code)
    merged_code, has_conflict = apply_patches(local_code, patches)
    
    # Prepend merged imports
    import_block = merge_imports(local_code, new_code)
    if import_block:
        merged_code = import_block + "\n\n" + merged_code

    if has_conflict:
        return False, f"[CONFLICT DETECTED]\n{merged_code}"

    try:
        ast.parse(merged_code)
    except SyntaxError as e:
        return False, f"Merge resulted in invalid syntax: {e.msg} at line {e.lineno}"

    return True, merged_code

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--new", required=True)
    parser.add_argument("--base", required=False, default=None)
    parser.add_argument("--project-root", required=False)
    args = parser.parse_args()
    
    with open(args.new, 'r', encoding='utf-8') as f:
        new_code = f.read()

    base_code = None
    if args.base and os.path.exists(args.base):
        with open(args.base, 'r', encoding='utf-8') as f:
            base_code = f.read()
        
    success, result = semantic_merge(args.target, new_code, base_code, args.project_root)
    
    if success:
        print("<<<<MERGE_SUCCESS>>>>")
        print(result)
        sys.exit(0)
    else:
        print(result, file=sys.stderr)
        sys.exit(1)
