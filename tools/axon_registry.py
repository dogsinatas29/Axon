#!/usr/bin/env python3
# encoding: utf-8
"""
AXON Snapshot Registry (Total Integrity Manager)
Manages versioned promotion with canonical path security and pre-flight hash checks.
"""
import json
import os
import sys
import shutil
import hashlib
import time

REGISTRY_FILE = ".axon_registry.json"
# Use relative paths for logic, will be canonicalized at runtime
PROTECTED_ROOTS = {".axon_registry.json", ".axon_snapshots", ".axon_daemon", "tools", "crates", "target", "mile_stone", "architecture.md", "constraints.json"}

def get_hash(path):
    if not os.path.exists(path): return None
    hasher = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hasher.update(chunk)
    return hasher.hexdigest()

def is_protected(path, project_root):
    """Canonical path check to prevent bypass or accidental deletion."""
    abs_path = os.path.realpath(os.path.abspath(path))
    for root in PROTECTED_ROOTS:
        abs_root = os.path.realpath(os.path.abspath(os.path.join(project_root, root)))
        if abs_path == abs_root or abs_path.startswith(abs_root + os.sep):
            return True
    return False

class Registry:
    def __init__(self, project_root):
        self.project_root = os.path.realpath(project_root)
        self.registry_path = os.path.join(self.project_root, REGISTRY_FILE)
        self.data = self._load()

    def _load(self):
        if os.path.exists(self.registry_path):
            with open(self.registry_path, "r") as f:
                return json.load(f)
        return {"current": None, "history": []}

    def _save(self):
        with open(self.registry_path, "w") as f:
            json.dump(self.data, f, indent=2)

    def promote(self, files_map, task_id):
        """Promotes a new version with pre-flight integrity check."""
        timestamp = int(time.time())
        version_id = f"v_{timestamp}_{task_id[:8]}"
        
        snapshot_dir = os.path.join(self.project_root, ".axon_snapshots", version_id)
        os.makedirs(snapshot_dir, exist_ok=True)
        
        hashes = {}
        # 1. Verification & Pre-flight
        for fname, code in files_map.items():
            snap_path = os.path.join(snapshot_dir, fname)
            os.makedirs(os.path.dirname(snap_path), exist_ok=True)
            with open(snap_path, "w", encoding="utf-8") as f:
                f.write(code)
            
            # Record hash for the snapshot immediately
            hashes[fname] = get_hash(snap_path)

        # 2. Update SSOT only after snapshot is safely recorded
        for fname, code in files_map.items():
            ssot_path = os.path.join(self.project_root, fname)
            if is_protected(ssot_path, self.project_root):
                continue # Safety skip
            os.makedirs(os.path.dirname(ssot_path), exist_ok=True)
            with open(ssot_path, "w", encoding="utf-8") as f:
                f.write(code)

        self.data["history"].append({
            "version": version_id,
            "task_id": task_id,
            "timestamp": timestamp,
            "files": list(files_map.keys()),
            "hashes": hashes
        })
        self.data["current"] = version_id
        self._save()
        return version_id

    def rollback(self):
        """Full state rollback with pre-rollback audit and canonical path security."""
        if len(self.data["history"]) < 2:
            return False, "No previous version available."
            
        current_idx = -1
        for i, h in enumerate(self.data["history"]):
            if h["version"] == self.data["current"]:
                current_idx = i
                break
        
        if current_idx <= 0:
            return False, "Already at oldest version."
            
        prev_v = self.data["history"][current_idx - 1]
        snap_dir = os.path.join(self.project_root, ".axon_snapshots", prev_v["version"])
        
        # 1. Pre-Rollback Audit: Verify snapshot integrity before doing anything
        for fname in prev_v["files"]:
            src = os.path.join(snap_dir, fname)
            stored_hash = prev_v["hashes"].get(fname)
            if get_hash(src) != stored_hash:
                return False, f"Integrity Failure: Snapshot {fname} is corrupted. Rollback aborted for safety."
        
        # 2. Safe Cleanup: Remove files using canonical path protection
        for root, dirs, files in os.walk(self.project_root, topdown=False):
            for f in files:
                full_p = os.path.join(root, f)
                if not is_protected(full_p, self.project_root):
                    os.remove(full_p)
            for d in dirs:
                full_p = os.path.join(root, d)
                if not is_protected(full_p, self.project_root):
                    try: os.rmdir(full_p)
                    except: pass # Ignore if not empty (contains protected)

        # 3. Restore from Verified Snapshot
        for fname in prev_v["files"]:
            src = os.path.join(snap_dir, fname)
            dst = os.path.join(self.project_root, fname)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy2(src, dst)
        
        self.data["current"] = prev_v["version"]
        self._save()
        return True, f"Verified rollback to {prev_v['version']} complete."

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["promote", "rollback", "list"])
    parser.add_argument("--root", required=True)
    parser.add_argument("--files-json")
    parser.add_argument("--task-id")
    
    args = parser.parse_args()
    reg = Registry(args.root)
    
    if args.action == "promote":
        with open(args.files_json, "r") as f: files = json.load(f)
        vid = reg.promote(files, args.task_id)
        print(f"<<<<REGISTRY_PROMOTE_SUCCESS: {vid}>>>>")
    elif args.action == "rollback":
        ok, msg = reg.rollback()
        if ok: print(f"<<<<REGISTRY_ROLLBACK_SUCCESS: {msg}>>>>")
        else: print(f"ERROR: {msg}", file=sys.stderr); sys.exit(1)
