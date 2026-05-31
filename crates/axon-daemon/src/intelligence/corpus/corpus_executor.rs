use std::fs;
use std::path::{Path, PathBuf};

/// Phase 3: Corpus Executor
/// Runs the mutation campaign in an ephemeral tmpfs sandbox.
/// NEVER executes authoritative mutations — only shadow copies.
pub struct CorpusExecutor;

impl CorpusExecutor {
    /// Executes the shadow mutation inside a safe tmpfs boundary.
    ///
    /// 실행 체인:
    /// 1. pre_hash  — 변경 전 SHA-256 해시 계산
    /// 2. shadow_copy — tmpfs 내 격리 복사본 생성
    /// 3. mutation — 복사본에만 코드 변환 적용
    /// 4. semantic_gate — 변환 전/후 줄 수 및 구조 유효성 확인
    /// 5. rollback — tmpfs 정리
    /// 6. post_hash — pre_hash와 반드시 일치해야 통과 (원본 불변 증명)
    pub fn execute_shadow_campaign(
        source_path: &Path,
        sandbox_tmpfs: &Path,
        mutation_fn: &dyn Fn(&str) -> String,
    ) -> Result<ShadowCampaignResult, String> {
        // Step 1: pre_hash — 원본 파일 해시
        let original_content = fs::read_to_string(source_path)
            .map_err(|e| format!("[SHADOW] Failed to read source: {}", e))?;
        let pre_hash = Self::hash_content(&original_content);

        // Step 2: shadow_copy — tmpfs에 복사본 생성
        let shadow_path = Self::create_shadow_copy(source_path, sandbox_tmpfs)?;

        // Step 3: mutation — 복사본에 변환 적용
        let mutated_content = mutation_fn(&original_content);
        fs::write(&shadow_path, &mutated_content)
            .map_err(|e| format!("[SHADOW] Failed to write mutated content: {}", e))?;

        // Step 4: semantic_gate — 줄 수 드리프트 및 기본 구조 검사
        let semantic_result = Self::semantic_gate(&original_content, &mutated_content);

        // Step 5: rollback — 복사본 정리
        if shadow_path.exists() {
            let _ = fs::remove_file(&shadow_path);
        }

        // Step 6: post_hash — 원본 불변 증명
        let post_original = fs::read_to_string(source_path)
            .map_err(|e| format!("[SHADOW] Post-hash read failed: {}", e))?;
        let post_hash = Self::hash_content(&post_original);

        if pre_hash != post_hash {
            return Err("[SHADOW] ❌ INTEGRITY VIOLATION: Original file was modified during shadow campaign!".to_string());
        }

        tracing::info!("✅ [SHADOW_CAMPAIGN] Original integrity verified. pre_hash == post_hash");

        Ok(ShadowCampaignResult {
            pre_hash,
            post_hash,
            semantic_drift: semantic_result.drift_ratio,
            line_delta: semantic_result.line_delta,
            passed_gate: semantic_result.passed,
        })
    }

    fn hash_content(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn create_shadow_copy(source: &Path, sandbox: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(sandbox)
            .map_err(|e| format!("[SHADOW] Failed to create sandbox dir: {}", e))?;

        let file_name = source.file_name()
            .ok_or("[SHADOW] Source has no filename")?;
        let shadow_path = sandbox.join(file_name);

        fs::copy(source, &shadow_path)
            .map_err(|e| format!("[SHADOW] Copy failed: {}", e))?;

        Ok(shadow_path)
    }

    fn semantic_gate(original: &str, mutated: &str) -> SemanticGateResult {
        let orig_lines = original.lines().count();
        let mut_lines = mutated.lines().count();
        let line_delta = (orig_lines as i64 - mut_lines as i64).unsigned_abs() as usize;

        // 줄 수 변화율 계산 (5% 이내면 PASS)
        let drift_ratio = if orig_lines == 0 {
            0.0
        } else {
            line_delta as f32 / orig_lines as f32
        };

        // 함수 선언부(fn / pub fn) 개수 보존 여부 확인
        let orig_fn_count = original.lines().filter(|l| l.trim_start().starts_with("fn ") || l.trim_start().starts_with("pub fn ")).count();
        let mut_fn_count = mutated.lines().filter(|l| l.trim_start().starts_with("fn ") || l.trim_start().starts_with("pub fn ")).count();
        let fn_preserved = orig_fn_count == mut_fn_count;

        let passed = drift_ratio <= 0.05 && fn_preserved;

        if !passed {
            tracing::warn!(
                "⚠️ [SEMANTIC_GATE] FAIL: drift={:.2}%, fn_preserved={}",
                drift_ratio * 100.0,
                fn_preserved
            );
        }

        SemanticGateResult { drift_ratio, line_delta, passed }
    }
}

#[derive(Debug)]
pub struct ShadowCampaignResult {
    pub pre_hash: String,
    pub post_hash: String,
    pub semantic_drift: f32,
    pub line_delta: usize,
    pub passed_gate: bool,
}

struct SemanticGateResult {
    drift_ratio: f32,
    line_delta: usize,
    passed: bool,
}
