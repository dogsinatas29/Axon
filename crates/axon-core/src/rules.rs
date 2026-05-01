use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureType {
    JsonParseError,
    MissingSymbol,
    SignatureMismatch,
    LanguageDrift,
    DependencyMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleScope {
    Local(String),   // project_id
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum Constraint {
    // Structural Invariants
    ExactFunctionExists { name: String },
    ExactSignatureMatch { name: String, args: Vec<String> },
    NoExtraFunctions,
    ReturnTypeConstraint { name: String, return_type: String },

    // Legacy/Custom
    NoJsonOutput,
    UseAxonPatchFormat,
    PythonOnly,
    MustImplementAllSymbols,
    NoMissingImports,
    Custom(String),
}

pub fn map_rule_to_constraint(rule: &Rule) -> Option<Constraint> {
    let c = rule.constraint.to_lowercase();

    if c.contains("do not output json") {
        return Some(Constraint::NoJsonOutput);
    }
    if c.contains("axon patch protocol") || c.contains("patch format") {
        return Some(Constraint::UseAxonPatchFormat);
    }
    if c.contains("only python") || c.contains("python syntax") {
        return Some(Constraint::PythonOnly);
    }
    if c.contains("function signatures") || c.contains("signature standard") {
        // Map to a generic custom constraint for now, or a specific one if possible
        return Some(Constraint::Custom("Signature Enforcement".to_string()));
    }
    if c.contains("implement all functions") || c.contains("all symbols") {
        return Some(Constraint::MustImplementAllSymbols);
    }
    if c.contains("missing imports") || c.contains("no hallucinated dependencies") {
        return Some(Constraint::NoMissingImports);
    }

    None
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Polarity {
    Positive, // Must do
    Negative, // Must NOT do
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub trigger: FailureType,
    pub constraint: String,
    pub polarity: Polarity,
    pub ir_constraint: Option<Constraint>, // Structured IR promotion
    
    // Scoring & Stats
    pub score: f32,
    pub confidence: f32,
    pub applied_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
    
    // Time tracking
    pub last_used: SystemTime,
    pub created_at: SystemTime,
    pub scope: RuleScope,
}

impl Rule {
    pub fn new(id: String, trigger: FailureType, constraint_str: String, scope: RuleScope) -> Self {
        let mut temp_rule = Self {
            id: id.clone(),
            trigger,
            constraint: constraint_str.clone(),
            polarity: Polarity::Negative,
            ir_constraint: None,
            score: 1.0,
            confidence: 0.5,
            applied_count: 0,
            success_count: 0,
            failure_count: 0,
            last_used: SystemTime::now(),
            created_at: SystemTime::now(),
            scope,
        };

        temp_rule.ir_constraint = map_rule_to_constraint(&temp_rule);
        temp_rule
    }

    pub fn compute_score(&self) -> f32 {
        let now = SystemTime::now();
        let base = 1.0;
        let frequency = (self.applied_count as f32).ln_1p(); // log scaling
        
        let success_rate = if self.applied_count > 0 {
            self.success_count as f32 / self.applied_count as f32
        } else {
            0.5
        };
        
        let success_boost = success_rate * 2.0;
        let failure_penalty = (self.failure_count as f32) * 0.5;
        
        let age = now.duration_since(self.last_used).unwrap_or_default().as_secs_f32();
        let decay_penalty = age / 3600.0 * 0.1; // Decay by 0.1 per hour
        
        base + frequency + success_boost - failure_penalty - decay_penalty
    }

    pub fn update_feedback(&mut self, success: bool) {
        self.applied_count += 1;
        self.last_used = SystemTime::now();
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
        self.confidence = self.success_count as f32 / self.applied_count as f32;
        self.score = self.compute_score();
    }

    pub fn should_promote(&self) -> bool {
        self.success_count >= 5 && 
        self.confidence > 0.7 && 
        matches!(self.scope, RuleScope::Local(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRegistry {
    pub rules: Vec<Rule>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, mut rule: Rule) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            // Update existing rule if same ID
            existing.score += 0.5; // Manual boost for recurring failures
        } else {
            rule.score = rule.compute_score();
            self.rules.push(rule);
        }
        self.sort_rules();
    }

    pub fn sort_rules(&mut self) {
        self.rules.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn prune(&mut self) {
        let now = SystemTime::now();
        self.rules.retain(|r| {
            let score = r.compute_score();
            let age = now.duration_since(r.last_used).unwrap_or_default().as_secs();
            
            // Hard pruning: remove low score or extremely old unused rules
            score > 0.5 && r.failure_count < 10 && age < 604800 // 7 days TTL
        });
    }

    pub fn select_top_k(&self, k: usize, project_id: &str) -> Vec<Rule> {
        let mut candidates: Vec<Rule> = self.rules.iter()
            .filter(|r| match &r.scope {
                RuleScope::Global => true,
                RuleScope::Local(pid) => pid == project_id,
            })
            .cloned()
            .collect();

        candidates.sort_by(|a, b| {
            // Priority: Local > Global, then by score
            let a_is_local = matches!(a.scope, RuleScope::Local(_));
            let b_is_local = matches!(b.scope, RuleScope::Local(_));
            
            if a_is_local != b_is_local {
                b_is_local.cmp(&a_is_local)
            } else {
                b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        candidates.into_iter().take(k).collect()
    }

    pub fn get_active_constraints(&self, k: usize, project_id: &str) -> String {
        let top_rules = self.select_top_k(k, project_id);
        if top_rules.is_empty() {
            return String::new();
        }

        let mut constraints = String::from("### SYSTEM CONSTRAINTS (EXPERIENCE-BASED) ###\n");
        for (i, rule) in top_rules.iter().enumerate() {
            constraints.push_str(&format!("{}. {}\n", i + 1, rule.constraint));
        }
        constraints.push_str("\n");
        constraints
    }

    pub fn learn_from_failure(&mut self, log: &str, project_id: &str) -> bool {
        let mut learned = false;
        let scope = RuleScope::Local(project_id.to_string());

        if log.contains("JSON Parse Error") || log.contains("Expecting value") {
            self.add_rule(Rule::new(
                "no_json_embedding".to_string(),
                FailureType::JsonParseError,
                "Do NOT embed code inside JSON. Use raw code blocks with AXON Patch Protocol v2.".to_string(),
                scope.clone(),
            ));
            learned = true;
        }

        if log.contains("\"use strict\"") || log.contains("const ") || log.contains("let ") {
            self.add_rule(Rule::new(
                "python_only".to_string(),
                FailureType::LanguageDrift,
                "Only Python syntax is allowed. No JavaScript or other languages.".to_string(),
                scope.clone(),
            ));
            learned = true;
        }

        if log.contains("[MISSING_SYMBOL]") || log.contains("[MISSING_COMPONENT]") {
            self.add_rule(Rule::new(
                "follow_spec_strictly".to_string(),
                FailureType::MissingSymbol,
                "Must implement ALL functions and components defined in architecture.md exactly.".to_string(),
                scope.clone(),
            ));
            learned = true;
        }

        learned
    }

    pub fn promote_rules(&mut self) {
        for rule in self.rules.iter_mut() {
            if rule.should_promote() {
                tracing::info!("🚀 Promoting Rule '{}' to GLOBAL scope.", rule.id);
                rule.scope = RuleScope::Global;
            }
        }
    }
}

pub struct RuleOptimizer;

impl RuleOptimizer {
    pub fn optimize(
        local: Vec<Rule>,
        global: Vec<Rule>,
        k: usize,
    ) -> Vec<Rule> {
        let combined = [local, global].concat();
        let mut optimized = Self::merge_rules(combined);
        optimized = Self::resolve_all_conflicts(optimized);
        
        // Final sorting by score
        optimized.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        optimized.into_iter().take(k).collect()
    }

    fn merge_rules(rules: Vec<Rule>) -> Vec<Rule> {
        let mut map: std::collections::HashMap<String, Rule> = std::collections::HashMap::new();

        for rule in rules {
            let key = Self::normalize_constraint(&rule.constraint);

            map.entry(key)
                .and_modify(|r: &mut Rule| {
                    r.score += rule.score;
                    r.applied_count += rule.applied_count;
                    r.success_count += rule.success_count;
                    r.failure_count += rule.failure_count;
                })
                .or_insert(rule);
        }

        map.into_values().collect()
    }

    fn normalize_constraint(s: &str) -> String {
        s.to_lowercase()
            .replace("never", "not")
            .replace("avoid", "not")
            .replace("return", "output")
            .trim()
            .to_string()
    }

    fn resolve_all_conflicts(mut rules: Vec<Rule>) -> Vec<Rule> {
        let mut to_remove = std::collections::HashSet::new();

        for i in 0..rules.len() {
            for j in i + 1..rules.len() {
                if Self::is_conflict(&rules[i], &rules[j]) {
                    if rules[i].score > rules[j].score {
                        to_remove.insert(j);
                        tracing::warn!("⚠️ Rule Conflict Detected: Dropping '{}' in favor of '{}'", rules[j].id, rules[i].id);
                    } else {
                        to_remove.insert(i);
                        tracing::warn!("⚠️ Rule Conflict Detected: Dropping '{}' in favor of '{}'", rules[i].id, rules[j].id);
                    }
                }
            }
        }

        let mut index = 0;
        rules.retain(|_| {
            let res = !to_remove.contains(&index);
            index += 1;
            res
        });

        rules
    }

    fn is_conflict(a: &Rule, b: &Rule) -> bool {
        // Simple conflict: same constraint, different polarity
        let norm_a = Self::normalize_constraint(&a.constraint);
        let norm_b = Self::normalize_constraint(&b.constraint);
        
        norm_a == norm_b && a.polarity != b.polarity
    }
}

pub struct FailureAnalyzer;

impl FailureAnalyzer {
    pub fn analyze(log: &str) -> Option<FailureType> {
        if log.contains("JSON Parse Error") || log.contains("Expecting value") {
            return Some(FailureType::JsonParseError);
        }

        if log.contains("[MISSING_SYMBOL]") || log.contains("[MISSING_COMPONENT]") {
            return Some(FailureType::MissingSymbol);
        }

        if log.contains("use strict") || log.contains("const ") || log.contains("let ") {
            return Some(FailureType::LanguageDrift);
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCluster {
    pub signature: String,
    pub count: u32,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStore {
    pub clusters: std::collections::HashMap<String, FailureCluster>,
}

impl ClusterStore {
    pub fn new() -> Self {
        Self { clusters: std::collections::HashMap::new() }
    }

    pub fn update(&mut self, log: &str) -> String {
        let sig = self.generate_signature(log);

        let entry = self.clusters.entry(sig.clone())
            .or_insert(FailureCluster {
                signature: sig.clone(),
                count: 0,
                last_seen: SystemTime::now(),
            });

        entry.count += 1;
        entry.last_seen = SystemTime::now();
        sig
    }

    fn generate_signature(&self, log: &str) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let norm = log.to_lowercase()
            .replace(|c: char| c.is_numeric(), "")
            .replace("\"", "")
            .replace("\n", " ");

        let mut hasher = DefaultHasher::new();
        norm.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn maybe_create_rule(&self, sig: &str, failure_type: FailureType, project_id: &str) -> Option<Rule> {
        if let Some(cluster) = self.clusters.get(sig) {
            if cluster.count >= 3 {
                let constraint = match failure_type {
                    FailureType::JsonParseError => "Do NOT embed code inside JSON. Use raw code blocks only.",
                    FailureType::LanguageDrift => "Only Python syntax allowed. No JavaScript keywords.",
                    FailureType::MissingSymbol => "Must implement all functions defined in architecture.md strictly.",
                    _ => "Follow the system specification exactly.",
                };

                return Some(Rule::new(
                    format!("rule_{}", sig),
                    failure_type,
                    constraint.to_string(),
                    RuleScope::Local(project_id.to_string()),
                ));
            }
        }
        None
    }
}
