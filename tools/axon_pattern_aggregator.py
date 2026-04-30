#!/usr/bin/env python3
# encoding: utf-8
import json
from collections import defaultdict
from datetime import datetime

def parse_ts(s):
    try:
        return datetime.fromisoformat(s.replace("Z", "+00:00"))
    except Exception:
        return None

def pattern_key(t):
    return (
        t.get("error"),
        t.get("file"),
        t.get("symbol"),
        t.get("module"),
    )

def load_standardized_traces(trace_paths):
    all_traces = []
    for path in trace_paths:
        if os.path.exists(path):
            with open(path, 'r', encoding='utf-8') as f:
                for line in f:
                    try:
                        t = json.loads(line)
                        # Ensure standardization
                        if "project" not in t: t["project"] = Path(path).parent.parent.name
                        if "trace_id" not in t: t["trace_id"] = "legacy-" + t.get("task_id", "unknown")
                        all_traces.append(t)
                    except: continue
    return all_traces

def aggregate(traces):
    agg = defaultdict(lambda: {
        "count": 0,
        "tasks": set(),
        "projects": set(),
        "stages": set(),
        "success_count": 0,
        "fail_count": 0,
        "trace_ids": []
    })

    # (Task outcomes logic same as before)
    task_outcomes = {}
    for t in traces:
        if t.get("error") == "TASK_SUMMARY":
            task_outcomes[t.get("task_id")] = t.get("file")

    total_tasks = len(set(t.get("task_id") for t in traces if t.get("task_id")))

    for t in traces:
        if t.get("error") == "TASK_SUMMARY": continue

        k = pattern_key(t)
        rec = agg[k]
        task_id = t.get("task_id")
        project = t.get("project", "unknown")

        rec["count"] += 1
        rec["projects"].add(project)
        if "stage" in t: rec["stages"].add(t["stage"])
        if "trace_id" in t: rec["trace_ids"].append(t["trace_id"])

        if task_id:
            rec["tasks"].add(task_id)
            outcome = task_outcomes.get(task_id)
            if outcome == "SUCCESS": rec["success_count"] += 1
            elif outcome == "FAIL": rec["fail_count"] += 1

    # Convert to enriched list with Explainability
    out = []
    for k, v in agg.items():
        error, file, symbol, module = k
        count = v["count"]
        v_fail = v["fail_count"]
        
        fpr = v["success_count"] / count if count > 0 else 0
        coverage = len(v["projects"])
        
        # Conservative Mining Logic (Rule 4)
        status = "CANDIDATE"
        if count >= 30 and coverage >= 3:
            if fpr <= 0.1: status = "HARD"
            elif fpr <= 0.2: status = "SOFT"
        
        out.append({
            "error": error, "file": file, "symbol": symbol, "module": module,
            "count": count, "status": status,
            "metrics": {"fpr": round(fpr, 2), "coverage": coverage},
            "evidence": {
                "projects": list(v["projects"]),
                "total_violations": count,
                "stages": list(v["stages"]),
                "sample_traces": v["trace_ids"][:3] # For drill-down (Rule 12)
            }
        })
    return out

def calculate_rule_score(m):
    # m: eval_count, violation_count, weighted_override_count
    eval_count = m.get("eval_count", 1)
    violation_count = m.get("violation_count", 0)
    w_override_count = m.get("weighted_override_count", 0)
    
    coverage = violation_count / max(eval_count, 1)
    override_rate = w_override_count / max(violation_count, 1)
    
    score = (0.6 * coverage) - (0.8 * override_rate)
    
    # Sample Size Penalty (Wilson-ish)
    if eval_count < 200:
        score *= 0.5
        
    return max(0.0, min(1.0, score))

def transition_state(current_state, metrics):
    orate = metrics["weighted_override_count"] / max(metrics["violation_count"], 1)
    
    if metrics["eval_count"] < 200:
        return current_state # Insufficient data
        
    if orate > 0.35: return "DEMOTED"
    if orate > 0.25: return "SOFTENED"
    if orate > 0.15: return "WATCH"
    if orate < 0.05: return "ACTIVE"
    
    return current_state

def aggregate(traces):
    # (Trace collection logic remains same...)
    human_signals = load_human_signals()
    
    OVERRIDE_WEIGHT = {
        "MUTE_RULE": 1.0,
        "FORCE_ALLOW": 0.8,
        "FREEZE": 0.3,
        "ADJUST_THRESHOLD": 0.5
    }
    
    # ... (Metrics calculation remains same...)
    for k, v in agg.items():
        p_id = str(k)
        h_sig = human_signals.get(p_id, {"mute_count": 0, "fp_count": 0})
        
        # Calculate weighted override count
        w_override = (h_sig["mute_count"] * OVERRIDE_WEIGHT["MUTE_RULE"]) + \
                     (h_sig["fp_count"] * OVERRIDE_WEIGHT["FORCE_ALLOW"])
        
        score = calculate_rule_score({
            "eval_count": total_tasks, # Simplified proxy for eval_count
            "violation_count": v["count"],
            "weighted_override_count": w_override
        })
        
        # Determine Status via State Machine
        status = transition_state("ACTIVE", {
            "eval_count": total_tasks,
            "violation_count": v["count"],
            "weighted_override_count": w_override
        })
        
        # Cross-project guard: if coverage is low, don't go HARD
        if status == "ACTIVE" and len(v["projects"]) < 3:
            status = "SOFT"
        
        out.append({
            "error": error, "file": file, "symbol": symbol, "module": module,
            "count": v["count"], "status": status, "score": round(score, 2),
            "metrics": {
                "fpr": round(v["success_count"] / v["count"] if v["count"] > 0 else 0, 2),
                "override_rate": round(w_override / max(v["count"], 1), 2)
            },
            "evidence": {
                "human_signals": h_sig,
                "weighted_override": round(w_override, 2)
            }
        })
    return out

    # 4. Conflict Detection
    conflicts = []
    from itertools import combinations
    for k1, k2 in combinations(enriched_agg.keys(), 2):
        p1 = enriched_agg[k1]
        p2 = enriched_agg[k2]
        
        if p1["file"] != p2["file"] or p1["count"] < 10 or p2["count"] < 10:
            continue
            
        tasks1 = p1["tasks"]
        tasks2 = p2["tasks"]
        common_tasks = tasks1 & tasks2
        
        if not common_tasks and p1["count"] > 20 and p2["count"] > 20:
            # Mutual Exclusivity - Hard Conflict Candidate
            conflicts.append({
                "type": "MUTUAL_EXCLUSIVITY",
                "rules": [str(k1), str(k2)],
                "score": 1.0,
                "classification": "HARD_CONFLICT"
            })
        elif common_tasks:
            # Behavioral Conflict: Opposite Impact
            impact_diff = abs(p1["metrics"]["impact"] - p2["metrics"]["impact"])
            if impact_diff > 0.5:
                conflicts.append({
                    "type": "OPPOSITE_IMPACT",
                    "rules": [str(k1), str(k2)],
                    "score": impact_diff,
                    "classification": "SOFT_CONFLICT"
                })

    return {"patterns": out, "conflicts": conflicts}

def generate_profiles(agg_res):
    patterns = agg_res["patterns"]
    conflicts_list = agg_res["conflicts"]
    
    # 1. Group by Type and Core Scope (Simplest Clustering)
    groups = defaultdict(list)
    for p in patterns:
        if p["status"] in ["SOFT", "HARD"]:
            # Categorize by error type and root directory or major file
            scope_key = p["file"].split('/')[0] if '/' in p["file"] else "root"
            key = (p["error"], scope_key)
            groups[key].append(p)
            
    # 2. Build Profiles from Groups
    suggested_profiles = []
    for key, cluster in groups.items():
        if len(cluster) < 3: # Size constraint
            continue
            
        # 3. Conflict Filtering within cluster
        # (Simplified: if any pair in cluster has a hard conflict, prune)
        # For now, we assume suggest_rules handled individual conflicts,
        # but profiles need higher integrity.
        
        # 4. Cluster Scoring
        avg_impact = sum(p["metrics"]["impact"] for p in cluster) / len(cluster)
        avg_fpr = sum(p["metrics"]["fpr"] for p in cluster) / len(cluster)
        
        score = avg_impact * (1.0 - avg_fpr) * (len(cluster) ** 0.5)
        
        if avg_impact >= 0.3 and avg_fpr <= 0.3:
            suggested_profiles.append({
                "profile_id": f"profile_{key[0].lower()}_{key[1].lower()}",
                "rule_count": len(cluster),
                "avg_impact": round(avg_impact, 2),
                "avg_fpr": round(avg_fpr, 2),
                "score": round(score, 2),
                "rules": [str((p["error"], p["file"], p.get("symbol"))) for p in cluster]
            })
            
    return suggested_profiles

def suggest_rules(agg_res):
    patterns = agg_res["patterns"]
    conflicts_list = agg_res["conflicts"]
    
    # 1. Build Conflict Graph
    adj = defaultdict(set)
    for c in conflicts_list:
        r1, r2 = c["rules"][0], c["rules"][1]
        adj[r1].add(r2)
        adj[r2].add(r1)

    # 2. Candidate Selection (Basic Threshold)
    candidates = []
    for p in patterns:
        m = p["metrics"]
        if p["count"] >= 30 and m["impact"] >= 0.4 and m["fpr"] <= 0.2:
            candidates.append(p)
            
    # 3. Sort Candidates (Better Function)
    # Impact (Desc) -> FPR (Asc) -> Coverage (Desc)
    candidates.sort(key=lambda x: (-x["metrics"]["impact"], x["metrics"]["fpr"], -x["metrics"]["coverage"]))

    # 4. Greedy Selection (Conflict-Free Subset)
    promoted_hard = set()
    blocked_by = {} # To track why someone was blocked
    
    for p in candidates:
        p_id = str((p["error"], p["file"], p["symbol"], p["module"]))
        
        # Check conflicts
        can_promote = True
        for neighbor in adj.get(p_id, []):
            if neighbor in promoted_hard:
                can_promote = False
                blocked_by[p_id] = neighbor
                break
        
        if can_promote:
            promoted_hard.add(p_id)

    # 5. Build Final Suggestions
    suggestions = []
    for p in patterns:
        p_id = str((p["error"], p["file"], p["symbol"], p["module"]))
        m = p["metrics"]
        
        status = p["status"]
        reason = None
        
        if p_id in promoted_hard:
            status = "HARD"
        elif p_id in blocked_by:
            status = "SOFT"
            reason = f"BLOCKED_BY_{blocked_by[p_id]}"
        elif p["status"] == "HARD": # Rollback if not in greedy selection anymore
            status = "SOFT"
            reason = "IMPACT_RANK_FALLBACK"

        if status in ["SOFT", "HARD"]:
            suggestions.append({
                "type": p["error"],
                "file": p.get("file"),
                "symbol": p.get("symbol"),
                "status": status,
                "reason": reason,
                "metrics": m,
                "occurrences": p["count"]
            })
            
    return suggestions
