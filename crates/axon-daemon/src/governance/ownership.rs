use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// P4: Immutable Skeleton Policy
/// Phase 1 skeleton is not editable or mutable randomly. It is strictly locked for structure.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkeletonPolicy {
    /// "구조는 고정, 내부 body만 제한적으로 개방"
    ImmutableUntilImplGen,
    /// 완전 봉쇄
    Sealed,
}

/// Semantic Identity representing the core signature without formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIdentity {
    pub name: String,
    pub visibility: String,
    pub return_contract: String,
    pub trait_binding: String,
    pub parameter_topology: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOwnership {
    pub symbol_id: String,
    pub owner_task_id: String,
    pub signature_hash: String,
    pub topology_hash: String,
    pub ownership_scope_hash: String,
    pub policy: SkeletonPolicy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnershipRegistry {
    pub symbols: HashMap<String, SymbolOwnership>,
}

impl OwnershipRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// P1: Skeleton Seal & P3: Ownership Lock
    /// "구현 이전에 topology를 먼저 국가 등록"
    pub fn register_symbol(
        &mut self,
        symbol_id: &str,
        owner_task_id: &str,
        raw_signature: &str,
    ) {
        let (canonical_sig, sig_hash) = Self::canonicalize_and_hash(raw_signature);
        let topology_hash = Self::hash_string(&format!("{}_{}", symbol_id, owner_task_id));
        let scope_hash = Self::hash_string(&canonical_sig);

        let ownership = SymbolOwnership {
            symbol_id: symbol_id.to_string(),
            owner_task_id: owner_task_id.to_string(),
            signature_hash: sig_hash,
            topology_hash,
            ownership_scope_hash: scope_hash,
            policy: SkeletonPolicy::ImmutableUntilImplGen, // P4 Immutable Policy
        };

        self.symbols.insert(symbol_id.to_string(), ownership);
    }

    /// P2: Signature Hash Freeze 
    /// "formatting, whitespace, comment가 아닌 semantic skeleton freeze"
    pub fn canonicalize_and_hash(raw_sig: &str) -> (String, String) {
        let mut normalized = String::new();
        let mut in_space = false;
        
        // 아주 단순한 정규화: 연속된 공백 및 줄바꿈을 단일 공백으로 압축
        for c in raw_sig.chars() {
            if c.is_whitespace() {
                if !in_space {
                    normalized.push(' ');
                    in_space = true;
                }
            } else {
                normalized.push(c);
                in_space = false;
            }
        }
        let canonical = normalized.trim().to_string();
        let hash = Self::hash_string(&canonical);
        (canonical, hash)
    }

    fn hash_string(s: &str) -> String {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// P3: Ownership Lock Checker
    /// "타 태스크 수정 금지, topology drift 금지, signature mutation 금지"
    pub fn can_mutate_body(&self, symbol_id: &str, task_id: &str) -> Result<(), String> {
        if let Some(ownership) = self.symbols.get(symbol_id) {
            if ownership.owner_task_id != task_id {
                return Err(format!("OWNERSHIP_VIOLATION: Symbol '{}' is owned by Task '{}'. Task '{}' is forbidden from mutating it.", symbol_id, ownership.owner_task_id, task_id));
            }
            if ownership.policy == SkeletonPolicy::Sealed {
                return Err(format!("OWNERSHIP_VIOLATION: Symbol '{}' is SEALED. No mutations allowed.", symbol_id));
            }
            Ok(())
        } else {
            Err(format!("OWNERSHIP_VIOLATION: Symbol '{}' has no registered owner. Unregistered topology cannot be mutated.", symbol_id))
        }
    }
}

/// P5: Skeleton Replay Snapshot
/// "동일 spec, 동일 replay seed에서 동일 skeleton graph가 나오는지 검증"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonReplaySnapshot {
    pub spec_hash: String,
    pub replay_seed: String,
    pub ownership_snapshot: OwnershipRegistry,
    pub topology_snapshot_json: String,
}

impl SkeletonReplaySnapshot {
    pub fn save(&self, project_root: &str) -> std::io::Result<()> {
        let path = std::path::Path::new(project_root).join("contracts/ownership_snapshot.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_sovereignty() {
        // Phase B - Ownership Sovereignty 검증
        let mut registry = OwnershipRegistry::new();
        let symbol = "pub fn parse_user(id: u32)";
        
        // Task A가 심볼을 최초 점유(Registration)
        registry.register_symbol("parse_user", "Task_A", symbol);

        // Task A는 자신의 심볼 바디를 구현(ImplGen) 가능해야 함
        let result_a = registry.can_mutate_body("parse_user", "Task_A");
        assert!(result_a.is_ok(), "Task A should be able to mutate its own symbol body");

        // Task B가 Task A의 심볼을 수정하려고 시도 -> 즉각 차단
        let result_b = registry.can_mutate_body("parse_user", "Task_B");
        assert!(result_b.is_err(), "Task B should NOT be able to mutate Task A's symbol");
        
        let err_msg = result_b.unwrap_err();
        assert!(err_msg.contains("OWNERSHIP_VIOLATION"));
        assert!(err_msg.contains("Task 'Task_B' is forbidden"));
    }
}
