import json
import os
from collections import defaultdict

class AxonRuleClusterer:
    def __init__(self, trace_log_path, profile_registry_path):
        self.trace_log_path = trace_log_path
        self.profile_registry_path = profile_registry_path
        self.min_cluster_size = 5

    def extract_features(self, trace):
        """Feature Extraction: Normalize raw trace into a consistent vector."""
        return {
            "rule": trace.get("rule_id", "unknown"),
            "file_type": os.path.splitext(trace.get("file", "unknown"))[1].replace(".", ""),
            "pattern": trace.get("context", {}).get("pattern", "unknown"),
            "stage": trace.get("stage", "unknown")
        }

    def cluster_traces(self):
        """Rule-centric Grouping logic."""
        if not os.path.exists(self.trace_log_path):
            return {}

        clusters = defaultdict(list)
        with open(self.trace_log_path, 'r') as f:
            for line in f:
                try:
                    trace = json.loads(line)
                    if trace.get("type") != "RULE_VIOLATION":
                        continue
                    
                    features = self.extract_features(trace)
                    cluster_id = f"{features['rule']}:{features['file_type']}"
                    clusters[cluster_id].append(features)
                except Exception:
                    continue

        return clusters

    def score_clusters(self, clusters):
        """Cluster Scoring: Identify meaningful patterns."""
        candidates = []
        for cluster_id, traces in clusters.items():
            count = len(traces)
            if count >= self.min_cluster_size:
                # Basic frequency-based candidate selection
                candidates.append({
                    "id": cluster_id,
                    "count": count,
                    "patterns": list(set(t["pattern"] for t in traces if t["pattern"] != "unknown"))
                })
        return candidates

    def generate_shadow_profiles(self, candidates):
        """Convert Clusters to Shadow Profiles."""
        shadow_profiles = {}
        for cand in candidates:
            profile_name = f"shadow_{cand['id'].replace(':', '_')}"
            shadow_profiles[profile_name] = {
                "status": "shadow",
                "source": "auto",
                "confidence": min(0.5 + (cand['count'] * 0.01), 0.95),
                "ir": {
                    "forbidden": {
                        "global": cand["patterns"]
                    }
                },
                "metrics": {
                    "violation_rate": 0.0, # Initial
                    "sample_size": cand["count"]
                }
            }
        return shadow_profiles

    def run(self):
        print("🔍 Starting Rule Clustering...")
        clusters = self.cluster_traces()
        candidates = self.score_clusters(clusters)
        new_shadows = self.generate_shadow_profiles(candidates)
        
        print(f"✅ Found {len(candidates)} candidates. Generated {len(new_shadows)} shadow profiles.")
        return new_shadows

if __name__ == "__main__":
    # Integration paths (placeholders for now)
    clusterer = AxonRuleClusterer("trace.log", "profiles.json")
    clusterer.run()
