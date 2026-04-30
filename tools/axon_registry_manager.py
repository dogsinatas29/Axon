import json
import os
import shutil
import hashlib

class AxonRegistryManager:
    def __init__(self, registry_root=".axon_registry"):
        self.root = registry_root
        self.global_dir = os.path.join(registry_root, "global/ir")
        self.projects_dir = os.path.join(registry_root, "projects")
        self._ensure_dirs()

    def _ensure_dirs(self):
        for d in [self.global_dir, self.projects_dir]:
            os.makedirs(d, exist_ok=True)

    def get_latest_version(self):
        versions = [v.replace(".json", "") for v in os.listdir(self.global_dir) if v.endswith(".json") and v != "current.json"]
        if not versions: return "0.0.0"
        return sorted(versions, key=lambda v: [int(x) for x in v.split('.')])[-1]

    def pull(self, project_id):
        """Sync Global IR to Project."""
        latest_v = self.get_latest_version()
        latest_path = os.path.join(self.global_dir, f"{latest_v}.json")
        project_path = os.path.join(self.projects_dir, project_id)
        os.makedirs(project_path, exist_ok=True)
        
        # Link current
        current_link = os.path.join(self.global_dir, "current.json")
        shutil.copy(latest_path, current_link)
        
        print(f"📥 Project '{project_id}' synced to Global IR v{latest_v}")

    def push(self, project_id, new_rules, message=""):
        """Promote Project rules to Global Registry."""
        latest_v = self.get_latest_version()
        latest_path = os.path.join(self.global_dir, f"{latest_v}.json")
        
        with open(latest_path, 'r') as f:
            global_ir = json.load(f)

        # Conflict Check & Merge
        for rule in new_rules:
            if self._has_conflict(rule, global_ir):
                print(f"❌ Conflict detected for rule '{rule['target']}'. Push rejected.")
                return False
            global_ir["rules"].append(rule)

        # Bump Version (Patch by default)
        v_parts = [int(x) for x in latest_v.split('.')]
        v_parts[2] += 1
        new_v = ".".join(map(str, v_parts))
        
        new_path = os.path.join(self.global_dir, f"{new_v}.json")
        global_ir["ir_version"] = new_v
        global_ir["meta"] = {"project": project_id, "message": message}
        
        with open(new_path, 'w') as f:
            json.dump(global_ir, f, indent=2)
            
        print(f"📤 Global IR bumped to v{new_v} by project '{project_id}'")
        return True

    def _has_conflict(self, rule, ir):
        for r in ir.get("rules", []):
            if r["target"] == rule["target"] and r["action"] != rule["action"]:
                return True
        return False

    def rollback(self, version):
        """Emergency Rollback to specific version."""
        target_path = os.path.join(self.global_dir, f"{version}.json")
        if not os.path.exists(target_path):
            print(f"❌ Version {version} not found.")
            return False
            
        current_link = os.path.join(self.global_dir, "current.json")
        shutil.copy(target_path, current_link)
        print(f"⏪ Global IR rolled back to v{version}")
        return True

    def detect_drift(self, project_id):
        # Implementation for drift detection based on hashes
        pass

if __name__ == "__main__":
    manager = AxonRegistryManager()
    # Initialize with 1.0.0 if empty
    if manager.get_latest_version() == "0.0.0":
        init_ir = {"ir_version": "1.0.0", "rules": [], "profiles": []}
        with open(os.path.join(manager.global_dir, "1.0.0.json"), "w") as f:
            json.dump(init_ir, f, indent=2)
        manager.pull("default")
