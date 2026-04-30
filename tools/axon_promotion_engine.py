import json
import os
import time
import hashlib
from collections import deque

class RuleCandidate:
    def __init__(self, key, target, action):
        self.key = key
        self.target = target
        self.action = action
        self.count = 0
        self.success_after = 0
        self.last_seen = time.time()

    def score(self):
        # Effectiveness: How often did this hint lead to success?
        effectiveness = self.success_after / max(1, self.count)
        # Recency: Exponential decay (simplified)
        recency = 1.0 / (1.0 + (time.time() - self.last_seen) / 3600)
        
        return (self.count * 0.5) + (effectiveness * 2.0) + (recency * 0.5)

class AxonPromotionEngine:
    def __init__(self, registry_path, trace_log_path):
        self.registry_path = registry_path
        self.trace_log_path = trace_log_path
        self.pool = {} # (key, target) -> RuleCandidate
        self.threshold = 5.0
        self.min_count = 3
        self.min_effectiveness = 0.6

    def ingest_from_logs(self):
        """Scan logs for RULE_VIOLATION and HINT_SUCCESS events."""
        if not os.path.exists(self.trace_log_path): return

        with open(self.trace_log_path, 'r') as f:
            for line in f:
                try:
                    event = json.loads(line)
                    # Extract violation to build candidate
                    if event.get("type") == "RULE_VIOLATION":
                        k = (event["error_type"], event["target"])
                        if k not in self.pool:
                            self.pool[k] = RuleCandidate(event["error_type"], event["target"], event["action"])
                        self.pool[k].count += 1
                        self.pool[k].last_seen = time.time()
                    
                    # Extract success to update effectiveness
                    elif event.get("type") == "HINT_SUCCESS":
                        k = (event["error_type"], event["target"])
                        if k in self.pool:
                            self.pool[k].success_after += 1
                except: continue

    def shadow_test(self, candidate, recent_traces):
        """Replay test: Does this rule actually reduce failure rate?"""
        # Simplified: Check if this candidate's target appeared in failures 
        # and stopped appearing after successes in the traces.
        return True # Placeholder for full replay logic

    def detect_conflict(self, candidate, ir):
        for existing in ir.get("rules", []):
            if candidate.target == existing["target"]:
                if candidate.action != existing["action"]:
                    return True
        return False

    def promote(self):
        print("⚖️ Starting Rule Promotion Trial...")
        self.ingest_from_logs()
        
        with open(self.registry_path, 'r') as f:
            ir = json.load(f)

        promoted = []
        for k, c in self.pool.items():
            if c.count >= self.min_count and (c.success_after / c.count) >= self.min_effectiveness:
                if c.score() >= self.threshold:
                    if not self.detect_conflict(c, ir):
                        print(f"🚀 Candidate {(c.key, c.target)} passed the trial. Promoting to IR.")
                        new_rule = {
                            "id": f"auto_{hashlib.md5(str(k).encode()).hexdigest()[:8]}",
                            "type": c.key,
                            "target": c.target,
                            "action": c.action,
                            "source": "auto",
                            "confidence": min(c.score() / 10.0, 0.95),
                            "evidence_count": c.count,
                            "created_at": int(time.time())
                        }
                        ir["rules"].append(new_rule)
                        promoted.append(k)

        if promoted:
            # Atomic update
            temp_path = self.registry_path + ".tmp"
            with open(temp_path, 'w') as f:
                json.dump(ir, f, indent=2)
            os.rename(temp_path, self.registry_path)
            print(f"✅ Successfully promoted {len(promoted)} rules to long-term IR.")

if __name__ == "__main__":
    engine = AxonPromotionEngine("constraints.json", "trace.log")
    engine.promote()
