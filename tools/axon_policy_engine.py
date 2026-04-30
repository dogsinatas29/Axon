import json
import os
import shutil
import hashlib

class AxonPolicyEngine:
    def __init__(self, registry_path, snapshot_dir):
        self.registry_path = registry_path
        self.snapshot_dir = snapshot_dir
        os.makedirs(snapshot_dir, exist_ok=True)

    def load_registry(self):
        if not os.path.exists(self.registry_path):
            return {"profiles": {}}
        with open(self.registry_path, 'r') as f:
            return json.load(f)

    def save_registry(self, data):
        with open(self.registry_path, 'w') as f:
            json.dump(data, f, indent=2)

    def take_snapshot(self, registry):
        """Rollback Mechanism: Save a snapshot before promotion."""
        reg_str = json.dumps(registry, sort_keys=True)
        reg_hash = hashlib.sha256(reg_str.encode()).hexdigest()[:8]
        snapshot_path = os.path.join(self.snapshot_dir, f"ruleset_{reg_hash}.json")
        
        with open(snapshot_path, 'w') as f:
            f.write(reg_str)
        return reg_hash

    def select_best_rule(self, rule_a, rule_b):
        """Decision Logic: Human > Priority > Confidence > FPR > Coverage."""
        # 1. Source (Human > Auto)
        if rule_a.get("source") != rule_b.get("source"):
            return rule_a if rule_a.get("source") == "human" else rule_b
        
        # 2. Priority
        p_a = rule_a.get("priority", 0)
        p_b = rule_b.get("priority", 0)
        if p_a != p_b:
            return rule_a if p_a > p_b else rule_b

        # 3. Confidence
        c_a = rule_a.get("confidence", 0.0)
        c_b = rule_b.get("confidence", 0.0)
        if abs(c_a - c_b) > 0.05: # Threshold
            return rule_a if c_a > c_b else rule_b

        # 4. Metrics: False Positive Rate (Lower is better)
        fpr_a = rule_a.get("metrics", {}).get("false_positive_rate", 1.0)
        fpr_b = rule_b.get("metrics", {}).get("false_positive_rate", 1.0)
        if fpr_a != fpr_b:
            return rule_a if fpr_a < fpr_b else rule_b

        # 5. Coverage (Higher is better)
        cov_a = rule_a.get("metrics", {}).get("coverage", 0)
        cov_b = rule_b.get("metrics", {}).get("coverage", 0)
        return rule_a if cov_a >= cov_b else rule_b

    def detect_conflicts(self, profiles):
        """Simple Conflict Detection: Overlapping forbidden words or required files."""
        conflicts = []
        # Logic to be expanded: check IR intersections
        return conflicts

    def promote_shadow_profiles(self):
        registry = self.load_registry()
        promoted_count = 0
        
        # Take snapshot before changes
        self.take_snapshot(registry)

        for name, profile in list(registry["profiles"].items()):
            if profile.get("status") == "shadow":
                metrics = profile.get("metrics", {})
                confidence = profile.get("confidence", 0.0)
                fpr = metrics.get("false_positive_rate", 0.0)
                sample_size = metrics.get("sample_size", 0)

                # Promotion Conditions
                if confidence > 0.8 and fpr < 0.1 and sample_size > 20:
                    print(f"🚀 Promoting shadow profile '{name}' to active.")
                    profile["status"] = "active"
                    promoted_count += 1

        if promoted_count > 0:
            self.save_registry(registry)
        
        return promoted_count

    def rollback(self, snapshot_hash):
        snapshot_path = os.path.join(self.snapshot_dir, f"ruleset_{snapshot_hash}.json")
        if os.path.exists(snapshot_path):
            shutil.copy(snapshot_path, self.registry_path)
            print(f"⏪ Rollback successful to snapshot {snapshot_hash}")
            return True
        return False

if __name__ == "__main__":
    engine = AxonPolicyEngine("profiles.json", "snapshots")
    engine.promote_shadow_profiles()
