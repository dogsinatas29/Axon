#!/usr/bin/env python3
# encoding: utf-8
import json
import os
import sys
from collections import defaultdict
from pathlib import Path

class ThresholdManager:
    def __init__(self, target_fp=0.05, step=0.01):
        self.thresholds = defaultdict(lambda: {"fpr": 0.2, "impact": 0.4, "latency": 200})
        self.target_fp = target_fp
        self.step = step

    def get_thresholds(self, profile_id):
        return self.thresholds[profile_id]

    def update_from_feedback(self, profile_id, feedback_type):
        """
        Adjusts thresholds based on human or automated feedback.
        """
        t = self.thresholds[profile_id]
        if feedback_type == "FALSE_POSITIVE":
            # Increase threshold to be more conservative
            t["fpr"] = min(0.5, t["fpr"] + self.step)
            t["impact"] = min(0.8, t["impact"] + self.step)
        elif feedback_type == "FALSE_NEGATIVE":
            # Decrease threshold to be more sensitive
            t["fpr"] = max(0.05, t["fpr"] - self.step)
            t["impact"] = max(0.2, t["impact"] - self.step)
        
        print(f"🔄 [THRESHOLD_TUNER] {profile_id} updated: FPR={t['fpr']:.2f}, Impact={t['impact']:.2f}")

class RiskScorer:
    def __init__(self, threshold_manager):
        self.tm = threshold_manager
        self.metrics_history = defaultdict(list)

    def calculate_score(self, profile_id, current_fpr, latency=0):
        t = self.tm.get_thresholds(profile_id)
        
        # Acceleration
        hist = self.metrics_history[profile_id]
        accel = 0
        if len(hist) >= 2:
            accel = (current_fpr - hist[-1]) - (hist[-1] - hist[-2])
        self.metrics_history[profile_id].append(current_fpr)

        score = 0
        if current_fpr > t["fpr"]: score += 2
        if current_fpr > t["fpr"] * 1.5: score += 4
        if accel > 0.05: score += 3
        if latency > t["latency"]: score += 1

        return score

class OverrideManager:
    def __init__(self, override_path="overrides.json"):
        self.path = Path(override_path)
        self.overrides = self._load()

    def _load(self):
        if self.path.exists():
            with open(self.path, 'r', encoding='utf-8') as f:
                return json.load(f).get("overrides", [])
        return []

    def get_override(self, profile_id, project_id=None):
        for o in self.overrides:
            if o.get("scope") == "global": return o
            if o.get("scope") == "profile" and o.get("target") == profile_id: return o
            if o.get("scope") == "project" and o.get("target") == project_id: return o
        return None

class ActionEngine:
    def __init__(self, history_depth=3):
        self.risk_history = defaultdict(list)
        self.history_depth = history_depth
        self.om = OverrideManager()

    def evaluate(self, profile_id, current_score, context=None):
        # 1. Check Human Override First (Supreme Authority)
        override = self.om.get_override(profile_id)
        if override:
            return override["action"], {"reason": "HUMAN_OVERRIDE", "actor": override.get("actor"), "trace": "MANUAL_COMMAND"}

        # 2. System Logic
        hist = self.risk_history[profile_id]
        hist.append(current_score)
        if len(hist) > self.history_depth: hist.pop(0)

        decision = "CONTINUE"
        reason = "Stable indicators"
        
        if len(hist) == self.history_depth and all(s >= 6 for s in hist):
            decision = "PREDICTIVE_ROLLBACK"
            reason = f"High risk score ({current_score}) sustained over {self.history_depth} windows"
        elif current_score >= 3:
            decision = "FREEZE_ROLLOUT"
            reason = f"Anomalous score ({current_score}) detected, freezing for safety"
            
        return decision, {"reason": reason, "trace": hist[:]}

    def trigger(self, action_type, profile_id, info):
        if action_type == "CONTINUE": return
        
        # Decision Trace (Explainability)
        trace = {
            "profile": profile_id,
            "decision": action_type,
            "reason": info["reason"],
            "evidence": info["trace"],
            "timestamp": "now"
        }
        
        print(f"\n📢 [GOVERNANCE] {action_type} for {profile_id}")
        print(f"   └─ Reason: {info['reason']}")
        if "actor" in info: print(f"   └─ Authorizer: {info['actor']}")
        
        # Log to audit trail
        with open(".axon_trace/audit_log.ndjson", "a", encoding='utf-8') as f:
            f.write(json.dumps(trace) + "\n")

def analyze_live_health(trace_paths, feedback_log=None):
    # (Trace collection remains same...)
    traces = []
    # ...

def analyze_live_health(trace_paths, feedback_log=None):
    # (Trace collection logic remains same...)
    traces = []
    for path in trace_paths:
        if os.path.exists(path):
            with open(path, 'r', encoding='utf-8') as f:
                for line in f:
                    try: traces.append(json.loads(line))
                    except: continue

    stats = defaultdict(lambda: {"total": 0, "violations": 0, "success": 0, "latency_sum": 0})
    for t in traces:
        pid = t.get("profile", "unknown")
        stats[pid]["total"] += 1
        if t.get("event") == "VIOLATION": stats[pid]["violations"] += 1
        elif t.get("event") == "PASS": stats[pid]["success"] += 1
        stats[pid]["latency_sum"] += t.get("latency_ms", 0)

    tm = ThresholdManager()
    
    # Apply Feedback if available
    if feedback_log and os.path.exists(feedback_log):
        with open(feedback_log, 'r', encoding='utf-8') as f:
            for line in f:
                fb = json.loads(line)
                tm.update_from_feedback(fb["profile"], fb["type"])

    scorer = RiskScorer(tm)
    engine = ActionEngine()
    results = []

    for pid, s in stats.items():
        fpr = s["success"] / s["total"] if s["total"] > 0 else 0
        avg_lat = s["latency_sum"] / s["total"] if s["total"] > 0 else 0
        
        score = scorer.calculate_score(pid, fpr, avg_lat)
        action = engine.evaluate(pid, score)
        
        if action != "CONTINUE":
            engine.trigger(action, {"profile": pid, "score": score})
        
        results.append({
            "profile": pid,
            "score": score,
            "action": action,
            "metrics": {"fpr": round(fpr, 2), "latency": round(avg_lat, 1)},
            "thresholds": tm.get_thresholds(pid)
        })
    return results

if __name__ == "__main__":
    health = analyze_live_health([".axon_trace/traces.ndjson"])
    print(json.dumps(health, indent=2))
