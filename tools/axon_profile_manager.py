#!/usr/bin/env python3
# encoding: utf-8
import json
import os
import shutil
import hashlib
from pathlib import Path

class BreakingChangeError(Exception):
    pass

class ProfileManager:
    def __init__(self, base_dir=".axon_profiles", snapshot_dir=".axon_snapshots"):
        self.base_dir = Path(base_dir)
        self.snapshot_dir = Path(snapshot_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.snapshot_dir.mkdir(parents=True, exist_ok=True)

    def save_version(self, profile_id, version, data):
        """
        Saves a new immutable version of a profile.
        """
        p_dir = self.base_dir / profile_id
        p_dir.mkdir(parents=True, exist_ok=True)
        
        v_file = p_dir / f"{version}.json"
        if v_file.exists():
            # Immutability Check
            with open(v_file, 'r', encoding='utf-8') as f:
                old_data = json.load(f)
            if self._calc_checksum(old_data) != self._calc_checksum(data):
                raise PermissionError(f"Profile {profile_id}@{version} is immutable and cannot be modified.")
            return

        # Compatibility Check (Conservative: No rule removal)
        latest_v = self._get_latest_version(profile_id)
        if latest_v:
            with open(p_dir / f"{latest_v}.json", 'r', encoding='utf-8') as f:
                self._check_compatibility(json.load(f), data)

        data["checksum"] = self._calc_checksum(data)
        with open(v_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2)

    def _check_compatibility(self, old, new):
        old_rules = old.get("rules", {})
        new_rules = new.get("rules", {})
        for rtype, items in old_rules.items():
            if isinstance(items, list):
                # Ensure no removal of mandatory items
                removed = set(items) - set(new_rules.get(rtype, []))
                if removed:
                    raise BreakingChangeError(f"New version removes rules: {removed}")

    def _calc_checksum(self, data):
        # Calculate checksum excluding metadata if any
        raw = json.dumps(data, sort_keys=True).encode('utf-8')
        return hashlib.sha256(raw).hexdigest()

    def _get_latest_version(self, profile_id):
        p_dir = self.base_dir / profile_id
        if not p_dir.exists(): return None
        versions = sorted([f.stem for f in p_dir.glob("v*.json")])
        return versions[-1] if versions else None

    def create_snapshot(self, snapshot_id, bindings):
        s_dir = self.snapshot_dir / snapshot_id
        s_dir.mkdir(parents=True, exist_ok=True)
        
        with open(s_dir / "bindings.json", 'w', encoding='utf-8') as f:
            json.dump(bindings, f, indent=2)
        
        # Link profiles used in this snapshot
        # ... (implementation for locking versions)

if __name__ == "__main__":
    pm = ProfileManager()
    p_data = {"id": "db_strict", "rules": {"require_import": ["sqlite3"]}}
    pm.save_version("db_strict", "v1", p_data)
    print("Profile v1 saved.")
    
    # Try saving v2 with removal (Should fail)
    try:
        p_data_v2 = {"id": "db_strict", "rules": {"require_import": []}}
        pm.save_version("db_strict", "v2", p_data_v2)
    except BreakingChangeError as e:
        print(f"Caught expected breaking change: {e}")
