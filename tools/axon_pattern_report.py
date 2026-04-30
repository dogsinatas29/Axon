#!/usr/bin/env python3
# encoding: utf-8
import json
import sys
import os
from pathlib import Path
from axon_pattern_aggregator import aggregate, suggest_rules

def load_ndjson(path):
    if not os.path.exists(path):
        return
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
            except:
                continue

def top_patterns(agg, n=10):
    return sorted(agg, key=lambda x: x["count"], reverse=True)[:n]

def main(trace_file=".axon_trace/traces.ndjson", top_n=10):
    if not os.path.exists(trace_file):
        print(f"No trace file found at {trace_file}")
        return

    traces = list(load_ndjson(trace_file))
    if not traces:
        print("Trace file is empty.")
        return

    res = aggregate(traces)
    agg = res["patterns"]
    conflicts = res["conflicts"]
    
    suggestions = suggest_rules(res)
    profiles = generate_profiles(res)
    
    from axon_monitor import analyze_live_health
    health = analyze_live_health([".axon_trace/traces.ndjson"])

    print("\n🏥 AXON LIVE HEALTH & PREDICTIVE RISK DASHBOARD")
    print("=" * 110)
    print(f"{'PROFILE@VER':<35} | {'TOTAL':<6} | {'FPR':<5} | {'SCORE':<5} | {'PREDICTIVE ACTION'}")
    print("-" * 110)
    for h in health:
        action_str = h["action"]
        if action_str == "PREDICTIVE_ROLLBACK": action_str = "🛑 ROLLBACK"
        elif action_str == "FREEZE_ROLLOUT": action_str = "❄️  FREEZE"
        elif action_str == "CONTINUE": action_str = "✅ ALLOW"
            
        print(f"{h['profile']:<35} | {h['metrics'].get('total', 0):<6} | {h['metrics']['fpr']:<5.2f} | {h['score']:<5} | {action_str}")

    print("\n🚀 AXON TOP FAILURE PATTERNS (Promotion Engine)")
    print("=" * 110)
    print(f"{'#':<2} | {'KEY':<45} | {'COUNT':<5} | {'IMPACT':<6} | {'FPR':<5} | {'COV':<5} | {'STATUS'}")
    print("-" * 110)
    for i, p in enumerate(top, 1):
        key = f"{p['error']} {p.get('file') or ''} {p.get('symbol') or p.get('module') or ''}".strip()
        key_trimmed = (key[:42] + '..') if len(key) > 44 else key
        
        m = p["metrics"]
        status_str = p["status"]
        if status_str == "HARD": status_str = "HARD ⭐⭐⭐"
        elif status_str == "SOFT": status_str = "SOFT ⭐"
        elif status_str == "DEPRECATED": status_str = "DEP 🗑️"
            
        print(f"{i:02d} | {key_trimmed:<45} | {p['count']:<5} | {m['impact']:<6} | {m['fpr']:<5} | {m['coverage']:<5} | {status_str}")

    if conflicts:
        print("\n⚖️ AXON RULE CONFLICT DASHBOARD")
        print("=" * 110)
        print(f"{'TYPE':<20} | {'RULES INVOLVED':<60} | {'STATUS'}")
        print("-" * 110)
        for c in conflicts:
            rules_str = " vs ".join(c["rules"])
            rules_trimmed = (rules_str[:57] + '..') if len(rules_str) > 59 else rules_str
            print(f"{c['type']:<20} | {rules_trimmed:<60} | {c['classification']}")

    if profiles:
        print("\n📦 AXON SUGGESTED PROFILES (Cluster Analysis)")
        print("=" * 110)
        print(f"{'PROFILE_ID':<30} | {'RULES':<5} | {'IMPACT':<7} | {'FPR':<5} | {'SCORE'}")
        print("-" * 110)
        for pr in profiles:
            print(f"{pr['profile_id']:<30} | {pr['rule_count']:<5} | {pr['avg_impact']:<7} | {pr['avg_fpr']:<5} | {pr['score']}")
        
        # Persist profiles
        profile_path = Path(".axon_trace/suggested_profiles.json")
        profile_path.write_text(json.dumps({"suggested_profiles": profiles}, indent=2), encoding='utf-8')
        print(f"\n✅ Suggested profiles saved to {profile_path}")

    if suggestions:
        print("\n📋 [PHASE 1] SUGGESTED RULES (JSON)")
        print("-" * 80)
        suggestion_json = json.dumps({"suggested_rules": suggestions}, indent=2)
        print(suggestion_json)
        
        # Persist for soft rule engine
        suggestion_path = Path(".axon_trace/suggested_rules.json")
        suggestion_path.write_text(suggestion_json, encoding='utf-8')
        print(f"\n✅ Suggested rules saved to {suggestion_path}")
        print("-" * 80)
        print("NOTICE: These rules are NOT applied automatically. Use soft_rule_engine for observation.")

if __name__ == "__main__":
    import os
    main()
