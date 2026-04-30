import json
import os
from collections import deque

class AxonPropagator:
    def __init__(self, registry_path, graph_path):
        self.registry_path = registry_path
        self.graph_path = graph_path
        self.max_depth = 2

    def load_registry(self):
        if not os.path.exists(self.registry_path):
            return {"profiles": {}}
        with open(self.registry_path, 'r') as f:
            return json.load(f)

    def load_graph(self):
        """Loads a dependency graph. 
        Format: {'nodes': [...], 'edges': [['child', 'parent'], ...]}
        """
        if not os.path.exists(self.graph_path):
            return {"nodes": [], "edges": []}
        with open(self.graph_path, 'r') as f:
            return json.load(f)

    def get_upstream_parents(self, node, graph):
        """Find nodes that depend on (import) the current node."""
        parents = []
        for child, parent in graph.get("edges", []):
            if child == node:
                parents.append(parent)
        return parents

    def is_eligible(self, rule, target):
        """Checks if rule can propagate to target (Type & Language checks)."""
        # Simplification: Assume target is a file string.
        # Check rule type and target extension
        if not target.endswith(".py") and rule.get("type") == "import":
            return False
        return True

    def propagate_rule(self, rule_id, profile, graph):
        origin_scopes = profile.get("ir", {}).get("files", {}).keys()
        if not origin_scopes:
            return {}

        propagated = {}
        queue = deque([(scope, 0) for scope in origin_scopes])
        visited = set(origin_scopes)

        while queue:
            node, depth = queue.popleft()
            if depth >= self.max_depth:
                continue

            parents = self.get_upstream_parents(node, graph)
            for parent in parents:
                if parent in visited:
                    continue
                
                if self.is_eligible(profile, parent):
                    # Propagate as Shadow
                    propagated[parent] = "shadow"
                    queue.append((parent, depth + 1))
                    visited.add(parent)

        return propagated

    def run(self):
        print("🌿 Starting Selective Rule Propagation...")
        registry = self.load_registry()
        graph = self.load_graph()
        
        updated = False
        for p_id, profile in registry.get("profiles", {}).items():
            # Only propagate high-confidence, active rules
            if profile.get("status") == "active" and profile.get("confidence", 0.0) > 0.8:
                propagated_map = self.propagate_rule(p_id, profile, graph)
                if propagated_map:
                    print(f"✅ Propagated '{p_id}' to: {list(propagated_map.keys())}")
                    profile["propagated"] = propagated_map
                    updated = True

        if updated:
            with open(self.registry_path, 'w') as f:
                json.dump(registry, f, indent=2)
        
        return updated

if __name__ == "__main__":
    propagator = AxonPropagator("profiles.json", "graph.json")
    propagator.run()
