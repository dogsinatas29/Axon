#!/usr/bin/env python3
# encoding: utf-8
import json
import sys
from collections import defaultdict

class ConflictError(Exception):
    def __init__(self, rule_type, item, path1, path2):
        self.msg = f"CONFLICT: Rule '{rule_type}' for '{item}' contradicts between {path1} and {path2}"
        super().__init__(self.msg)

class ProfileResolver:
    def __init__(self, profiles_db, max_depth=4):
        self.db = profiles_db # profile_id -> data
        self.max_depth = max_depth

    def resolve(self, profile_refs, file_overrides=None):
        """
        Resolves a list of profile references (id@version) + file overrides.
        """
        final_rules = {
            "require_import": set(),
            "forbid_call": set(),
            "require_symbol": set(),
            "exclusive": {}
        }
        decision_trace = defaultdict(list)

        for ref in profile_refs:
            pid, version = self._parse_ref(ref)
            self._resolve_recursive(pid, version, final_rules, decision_trace, 0)
        # ... (rest of logic remains same)

    def _parse_ref(self, ref):
        if '@' in ref:
            return ref.split('@', 1)
        return ref, "v1" # Default to v1 if not specified

    def _load_profile(self, pid, version):
        # 1. Handle Org Namespace Mapping
        if pid.startswith("org::"):
            # org::domain::id -> org_domain_id
            parts = pid.replace("org::", "").split("::")
            if len(parts) == 2:
                pid = f"org_{parts[0]}_{parts[1]}"

        # 2. Check in-memory DB (from suggestions)
        if pid in self.db and self.db[pid].get("version") == version:
            return self.db[pid]
            
        # 3. Check filesystem (.axon_profiles local cache)
        path = Path(".axon_profiles") / pid / f"{version}.json"
        if path.exists():
            with open(path, 'r', encoding='utf-8') as f:
                return json.load(f)
        return None
        for parent_ref in profile.get("extends", []):
            ppid, pv = self._parse_ref(parent_ref)
            self._resolve_recursive(ppid, pv, current_rules, trace, depth + 1)

        path = f"profile:{pid}@{version}"
        # ... (rules application logic remains same)

        for imp in p_rules.get("require_import", []):
            current_rules["require_import"].add(imp)
            trace[f"import:{imp}"].append(path)

        for call in p_rules.get("forbid_call", []):
            current_rules["forbid_call"].add(call)
            trace[f"forbid:{call}"].append(path)

        for sym in p_rules.get("require_symbol", []):
            current_rules["require_symbol"].add(sym)
            trace[f"symbol:{sym}"].append(path)

        for k, v in p_rules.get("exclusive", {}).items():
            current_rules["exclusive"][k] = v
            trace[f"exclusive:{k}"].append(path)

    def _apply_overrides(self, overrides, current_rules, trace):
        path = "file_override"
        # Allow (Additive)
        for imp in overrides.get("allow", {}).get("require_import", []):
            current_rules["require_import"].add(imp)
            trace[f"import:{imp}"].append(path)
            
        # Deny (Removal)
        for imp in overrides.get("deny", {}).get("require_import", []):
            if imp in current_rules["require_import"]:
                current_rules["require_import"].remove(imp)
                trace[f"import:{imp}"].append(f"{path}:REMOVED")

    def _detect_hard_conflicts(self, rules, trace):
        # Example: if something is required AND forbidden
        # (This logic expands as we add more rule types)
        pass

if __name__ == "__main__":
    # Test logic
    db = {
        "base": {"rules": {"require_import": ["os"]}},
        "db": {"extends": ["base"], "rules": {"require_import": ["sqlite3"]}}
    }
    resolver = ProfileResolver(db)
    rules, trace = resolver.resolve(["db"], {"deny": {"require_import": ["os"]}})
    print(json.dumps({"rules": {k: list(v) if isinstance(v, set) else v for k,v in rules.items()}, "trace": trace}, indent=2))
