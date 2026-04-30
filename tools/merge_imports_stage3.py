import ast

def merge_imports(local_code: str, new_code: str) -> str:
    local_tree = ast.parse(local_code)
    new_tree = ast.parse(new_code)
    
    # Extract imports
    # Group by module for ImportFrom, or name for Import
    imports_map = {} # module -> set of names
    
    def process_tree(tree):
        for node in tree.body:
            if isinstance(node, ast.Import):
                for alias in node.names:
                    # Treat standard imports as from '' import name
                    imports_map.setdefault('', set()).add((alias.name, alias.asname))
            elif isinstance(node, ast.ImportFrom):
                mod = node.module or ''
                # handle relative imports? node.level
                prefix = '.' * node.level if node.level else ''
                mod_full = prefix + mod
                for alias in node.names:
                    imports_map.setdefault(mod_full, set()).add((alias.name, alias.asname))
                    
    process_tree(local_tree)
    process_tree(new_tree)
    
    # Reconstruct
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
            
    return "\n".join(lines)
