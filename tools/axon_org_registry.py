#!/usr/bin/env python3
# encoding: utf-8
import json
import os
from pathlib import Path

class OrgRegistry:
    def __init__(self, registry_root=".axon_org_registry"):
        self.root = Path(registry_root)
        self.profiles_dir = self.root / "profiles"
        self.index_path = self.root / "index.json"
        
        self.root.mkdir(parents=True, exist_ok=True)
        self.profiles_dir.mkdir(parents=True, exist_ok=True)
        
        if not self.index_path.exists():
            self._save_index({})

    def register_profile(self, domain, profile_id, version, data, status="candidate"):
        """
        Registers a new profile version in the organizational registry.
        Full ID: org::<domain>::<profile_id>@<version>
        """
        full_id = f"org::{domain}::{profile_id}"
        p_dir = self.profiles_dir / domain / profile_id
        p_dir.mkdir(parents=True, exist_ok=True)
        
        v_file = p_dir / f"{version}.json"
        with open(v_file, 'w', encoding='utf-8') as f:
            data["org_metadata"] = {
                "id": full_id,
                "version": version,
                "status": status,
                "tier": data.get("tier", 1) # Default to Tier 1
            }
            json.dump(data, f, indent=2)
            
        # Update Index
        index = self._load_index()
        if full_id not in index:
            index[full_id] = {"versions": [], "latest": version, "status": {}}
        
        if version not in index[full_id]["versions"]:
            index[full_id]["versions"].append(version)
            
        index[full_id]["status"][version] = status
        if status == "stable":
            index[full_id]["latest"] = version
            
        self._save_index(index)

    def pull_to_project(self, profile_ref, project_path=".axon_profiles"):
        """
        Pulls a specific profile version from registry to project local cache.
        """
        # Parsing: org::<domain>::<id>@<version>
        try:
            parts = profile_ref.replace("org::", "").split("::")
            domain = parts[0]
            id_ver = parts[1].split("@")
            pid = id_ver[0]
            version = id_ver[1]
            
            src = self.profiles_dir / domain / pid / f"{version}.json"
            if not src.exists():
                raise FileNotFoundError(f"Registry: Profile {profile_ref} not found.")
            
            dest_dir = Path(project_path) / f"org_{domain}_{pid}"
            dest_dir.mkdir(parents=True, exist_ok=True)
            
            shutil.copy(src, dest_dir / f"{version}.json")
            return True
        except Exception as e:
            print(f"Sync Fail: {e}")
            return False

    def _load_index(self):
        with open(self.index_path, 'r', encoding='utf-8') as f:
            return json.load(f)

    def _save_index(self, index):
        with open(self.index_path, 'w', encoding='utf-8') as f:
            json.dump(index, f, indent=2)

if __name__ == "__main__":
    import shutil
    reg = OrgRegistry()
    reg.register_profile("security", "no_rce", "v1", {
        "rules": {"forbid_call": ["os.system"]},
        "tier": 0
    }, status="stable")
    print("Org Profile Registered.")
