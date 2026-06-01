# encoding: utf-8

# 🏛️ AXON Dead Code Archaeology Report — v0.0.31

> **감사 일시:** 2026-06-02
> **기준:** `crates/axon-daemon/src/` 전수 `rg` 호출 그래프 조사
> **휴먼 분석:** "고고학 발굴" 단계 — 단순 코드 정리가 아닌 철학적 분류
> **참조:** [`GEMINI.md`](GEMINI.md) §Architecture Map — Layer Map, State Taxonomy
> **GEMINI.md 내 분석:** GEMINI.md §377-395에 Layer Map으로 요약됨

---

## 1. Executive Summary: 철학의 변화

이번 감사에서 가장 중요한 발견은 **단순한 데드코드 식별이 아니라 프로젝트 철학 자체의 변화**입니다.

| 시대 | 엔진 | 특징 | 상태 |
|------|------|------|------|
| **초기 AXON** (v0.0.1~v0.0.23) | Mutation Engine | 자가 진화, 변이 실험, Observe→Mutate→Validate→Commit 자동 루프 | 철폐 |
| **현재 AXON** (v0.0.24~v0.0.31) | Contract Engine | 인간 승인, 게이트키퍼, 진단서, `.failed.json` 영속화, Promotion Gate | 운용중 |

DEAD 목록 대부분은 단순 미구현이 아니라 **이전 철학의 유물**입니다. 따라서 삭제 판단은 기술적 대체재 존재 여부뿐 아니라, **현재 철학과의 정합성**을 기준으로 해야 합니다.

---

## 2. Classification Framework

GEMINI.md §Module State Taxonomy 기반 6가지 상태 + 사용자 분석 기반 2가지 처리 분류:

| 상태 | 의미 | 판단 기준 |
|------|------|-----------|
| **ACTIVE** | 실제 실행 경로에서 호출됨 | `execute_one_task()`에서 직접/간접 호출 확인 |
| **REACHABLE** | 현재 호출되지 않지만 파이프라인에 연결 가능 | mod.rs 선언 + builder/getter 패턴 |
| **DORMANT** | 현재 도달 불가능하지만 미래 기능으로 보존 가치 있음 | 완전 구현 + 대체재 없음 + v0.1.x 로드맵 연결 |
| **DEAD** | 호출 없음 + 의미 소멸 | 대체재 존재 또는 이전 철학 잔재 |
| **LEGACY** | 과거 시스템 호환, 메인 파이프라인 미연결 | legacy 전용 경로에서만 호출 |
| **PASSIVE** | 실행되지만 관측만 수행, 흐름 미차단 | 호출됨 + 결과 무시 (WARN-only) |

### 처리 분류

| 분류 | 처리 | 조건 |
|------|------|------|
| **즉시 제거 후보** | `quarantine/dead_a/` 이동 | DEAD + 대체재 명확 |
| **위험한 제거 후보** | `quarantine/research/` 이동 | DEAD + 연구 자산 가치 |
| **전략 자산** | 현재 위치 보존 | DORMANT + v0.1.x 활성화 예정 |
| **운용 자산** | 유지 | ACTIVE / REACHABLE |
| **레거시** | `quarantine/` 유지 | LEGACY |

---

## 3. 즉시 제거 후보 (DEAD + 대체재 명확)

6개 항목. 공통점: 호출처 없음, 테스트 없음, 문서 최신판 언급 없음, 더 강한 대체재 존재.

### 3.1 `IntelligenceEngine`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/intelligence/mod.rs:55` |
| **호출 여부** | ❌ |
| **대체재** | `execute_one_task()` (pipeline.rs) |
| **사망 원인** | Hub-and-Spoke 리팩토링(v0.0.31)에서 파사드로 설계되었으나, 실제 파이프라인은 `ExecutionPipeline::execute_one_task()`가 직접 제어. 이중 지휘 체계. |

### 3.2 `EvolutionPipeline`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/intelligence/pipeline.rs:13` |
| **호출 여부** | ❌ |
| **대체재** | `execute_one_task()` (pipeline.rs) |
| **사망 원인** | "자가 진화" 철학 — 현재 AXON은 "인간 승인 기반 진화". 철학 충돌. v0.0.23에서 구현되었으나 v0.0.31에서 완전히 대체됨. |

### 3.3 `on_validation_cycle()`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/intelligence/orchestrator.rs:45` |
| **호출 여부** | ❌ |
| **대체재** | `execute_one_task()` (pipeline.rs) |
| **사망 원인** | 과거 Observe→Mutate→Validate→Commit 자동 루프. 현재는 Observe→Propose→Human Approval→Commit. 철학 충돌. |

### 3.4 `MutationReplayObservatory`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/intelligence/observatory.rs:16` |
| **호출 여부** | ❌ |
| **대체재** | `FailedDiagnostic` + `.failed.json` + `PromotionDecision` |
| **사망 원인** | mutation 기록 기능이 `.failed.json` 진단서와 Board History로 완전히 대체됨. 세대교체 완료. |

### 3.5 `extract_undefined_symbols()`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/execution_validator.rs` |
| **호출 여부** | ❌ |
| **대체재** | `extract_error_locations()` |
| **사망 원인** | 컴파일러 에러 파싱의 레거시 함수. 현재는 `extract_error_locations()`가 모든 언어 포맷(C/Rust/Python/Lua)을 처리. |

### 3.6 `detect_senior_header_hallucination()`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/pipeline.rs` |
| **호출 여부** | ❌ (소스에 "폐기" 주석 명시) |
| **대체재** | `autonomous_header_slicer()` + Senior 확장자 힌트 프롬프트 |
| **사망 원인** | Senior 헤더 환각을 사후 탐지하던 함수. 현재는 사전 예방(파일 확장자 명시 + Integrator 노트)으로 대체. |

---

## 4. 위험한 제거 후보 → `quarantine/research/` 이동

2~4개 항목. 현재 DEAD로 보이나, 향후 기능 확장 시 유사 개념 재등장 가능성이 높아 **삭제보다 격리**가 안전.

### 4.1 `MutationCampaign` / `CampaignRunner`

| 항목 | 값 |
|------|-----|
| **위치** | `crates/axon-daemon/src/intelligence/mutation/` |
| **호출 여부** | ❌ |
| **대체재** | 없음 |
| **보존 사유** | AXON 초창기의 "변이 실험장" 계열. 향후 `CorpusExecutor::execute_shadow_campaign()`, Regression Corpus, Shadow Campaign을 강화할 계획이 있다면 다시 비슷한 개념이 등장할 가능성 높음. |

**권장:** `quarantine/research/`로 이동. 삭제는 보류.

### 4.2 Semantic 계열 (DEAD-B)

다음 항목들은 `RustCanonicalizer` 서브시스템의 보조 구조체로, 상위 시스템 전체가 고아 상태:

| 항목 | 위치 |
|------|------|
| `SemanticAuthorityGate` | `intelligence/semantic_authority_gate.rs` |
| `SemanticCanonicalizer` | `intelligence/canonicalizer.rs` |
| `SemanticDistance` | `intelligence/semantic_distance.rs` |
| `SemanticMutationClass` | `intelligence/semantic_severity.rs` |
| `SemanticSeverity` | `intelligence/semantic_severity.rs` |
| `SemanticTokens` | `intelligence/semantic_tokens.rs` |
| `RawAstNode` | `intelligence/canonicalizer.rs` |
| `CanonicalSemanticForm` | `intelligence/canonicalizer.rs` |
| `VisibilityScope` | `intelligence/` |
| `TopologyEdge` | `intelligence/` |
| `SignatureVector` | `intelligence/` |

**권장:** 11개 항목을 `quarantine/research/`로 일괄 이동. 시맨틱 정규화는 v0.1.x IR 기반 수술에서 재검토 가능.

---

## 5. DORMANT 전략 자산 (보존)

15개 항목. 현재 도달 불가능하지만 미래 기능으로 보존 가치가 높음. **절대 삭제 금지.**

### 5.1 `StructuralRoundtripValidator`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT |
| **가치** | 🟢 **전략 자산** |
| **활성화 조건** | AXON이 `Source → IR → Source` 라운드트립 경로를 사용하게 될 때 |
| **예상 시점** | v0.1.x (IR 기반 수술 도입 시) |

IR 기반 구조 단위 수술이 도입된다면, Source→IR 변환 후 역변환 시 정보 손실이 없는지 검증하는 라운드트립 검증기는 결국 필요합니다. 오히려 반드시 돌아옵니다.

### 5.2 `LowerInsertField` / `LowerAppendStmt` / `LowerReplaceFnBody`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT |
| **가치** | 🟢 **전략 자산** |
| **활성화 조건** | 파일 단위 수정 → 함수/블록/AST 단위 수정으로 진화할 때 |
| **예상 시점** | v0.1.x |

현재 AXON은 **파일 단위 수정** 위주입니다. 궁극적으로 함수 단위 → 블록 단위 → AST 단위 수정으로 가고 싶어할 가능성이 높습니다. 이 세 개가 가장 먼저 부활할 항목들입니다.

### 5.3 `SurgicalEditor` / `SurgicalReplayHarness`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT |
| **가치** | 🟢 **전략 자산** |
| **활성화 조건** | AST 단위 정밀 수술 도입 시 |
| **예상 시점** | v0.1.x |

`LowerInsertField` 계열이 "무엇을 수정할지" 정의한다면, `SurgicalEditor`는 "어떻게 수정할지" 실행합니다. 쌍으로 부활할 가능성 높음.

### 5.4 `CrashSandbox`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT |
| **가치** | 🟢 **전략 자산** |
| **활성화 조건** | 런타임 물리 센서/샌드박스 실행 계열 확장 시 |
| **예상 시점** | v0.1.x |

런타임 검증을 샌드박스에서 격리 실행하는 개념. 현재 `execution_validator::validate()`가 단순 컴파일 검증만 수행하므로, 크래시/메모리 검증으로 확장 시 필요.

### 5.5 AST 검증기 3종

| 항목 | 값 |
|------|-----|
| **항목** | `AstOwnershipValidator`, `RegexAstValidator`, `TreeSitterAstValidator` |
| **분류** | DORMANT |
| **가치** | 🟡 **보존** (중간) |
| **활성화 조건** | AST 단위 수술 또는 Tree-sitter 기반 정적 분석 강화 시 |

### 5.6 기타 DORMANT

| 항목 | 상태 | 비고 |
|------|------|------|
| `StabilityMatrixHarness` | 🟡 보존 | 안정성 매트릭스 자동 측정 |
| `RustCanonicalizer` | 🟡 보존 | Semantic Canonicalizer의 상위 — 단, 하위 구조체는 DEAD-B 처리 |
| `TreeSitterLocator` | 🟢 전략 | AST 노드 위치 정밀 탐색, SurgicalEditor 계열과 연동 |
| `AnchorValidator` | 🟢 전략 | 수정 앵커 검증, SurgicalEditor 계열과 연동 |
| `dual_run_shadow_validation` | 🟡 보존 | 그림자 실행 병렬 검증 |

### DORMANT 활성화 로드맵

```
v0.0.31 (현재)                          v0.1.x
─────────────────────────────────────────────────────────
파일 단위 수정 ──────────▶ 함수 단위 수정 ───▶ AST 단위 수정
                              │                    │
                              ├── LowerInsertField  ├── SurgicalEditor
                              ├── LowerAppendStmt   ├── TreeSitterLocator
                              ├── LowerReplaceFnBody├── AnchorValidator
                              └── StructuralRoundtrip
```

---

## 6. ACTIVE 현황 (19개)

Pipeline Core 19개 항목. 유지.

| # | 항목 | 파일 |
|---|------|------|
| 1 | `junior.process_task()` | pipeline.rs:1328 |
| 2 | `senior.review_proposal()` | pipeline.rs:1733 |
| 3 | `LspSupervisor::semantic_gate()` | pipeline.rs:1576 |
| 4 | `execution_validator::validate()` | pipeline.rs:1637 |
| 5 | `unlock_promotion()` | pipeline.rs:1922 |
| 6 | `save_failed_diagnostic()` | pipeline.rs (5개 지점) |
| 7 | `diagnostic_to_regions()` | pipeline.rs:1057 |
| 8 | `load_failed_regions()` | pipeline.rs:1099 |
| 9 | `validate_patch_radius()` | pipeline.rs:1683 |
| 10 | `check_forbidden_symbols()` | pipeline.rs:1470 |
| 11 | `check_allowed_includes()` | pipeline.rs:1505 |
| 12 | `CorpusExecutor::execute_shadow_campaign()` | pipeline.rs:1846 |
| 13 | `autonomous_header_slicer()` | pipeline.rs:1365 |
| 14 | `is_empty_or_comments_only()` | pipeline.rs:1448 |
| 15 | `TaxonomyMigrationManifest::build_v2()` | pipeline.rs:1538 |
| 16 | `PredictiveImmuneLayer::check_against_archive()` | pipeline.rs:1547 |
| 17 | `PromotionDecision` | pipeline.rs:1912 |
| 18 | `extract_error_locations()` | pipeline.rs:1647 |
| 19 | `FailedDiagnostic` | pipeline.rs (전체) |

---

## 7. PASSIVE / REACHABLE 현황

### PASSIVE (1개)

| 항목 | 상태 | 사유 |
|------|------|------|
| `validate_patch_radius()` | WARN-only | 의도적 데이터 수집 모드. 향후 AUTO_REJECT 격상 검토 |

### REACHABLE (2개)

| 항목 | 위치 | 사유 |
|------|------|------|
| `with_agent_pool()` | builder 패턴 | v0.0.32 동적 에이전트 풀 구성 시 활성화 예정 |
| `pending_reviews_handle()` | getter | 리뷰 큐 조회용 진입점 |

---

## 8. LEGACY (2개) — `quarantine/` 유지

| 항목 | 호출 위치 | 대체재 |
|------|-----------|--------|
| `selective_run()` | `quarantine/legacy_daemon.rs` | `validate()` |
| `extract_error_files()` | `quarantine/legacy_daemon.rs` | `extract_error_locations()` |

`quarantine/`은 컴파일 트리에서 배제된 아카이브 상태 유지. 참조용 보존.

---

## 9. Disposal Strategy & Timeline

### 통합 테이블

| 분류 | 항목 수 | 처리 | 시점 |
|------|---------|------|------|
| DEAD + 대체재 명확 | 6 | `quarantine/dead_a/` 이동 | Win32/GTK 검증 완료 후 |
| DEAD + 연구 자산 (Semantic 계열) | 11 | `quarantine/research/` 이동 | Win32/GTK 검증 완료 후 |
| DEAD + 연구 자산 (Mutation 계열) | 2 | `quarantine/research/` 이동 | Win32/GTK 검증 완료 후 |
| DORMANT + 전략 자산 | 15 | 현재 위치 보존 | v0.1.x 로드맵 연결 |
| ACTIVE | 19 | 유지 | — |
| LEGACY | 2 | `quarantine/` 유지 | — |

### 타임라인

| 단계 | 작업 | 조건 | 시점 |
|------|------|------|------|
| 1 | ✅ 감사 결과 문서화 (`DEAD_CODE_AUDIT.md`) | — | ✅ **완료** |
| 2 | DEAD-A 6개 → `quarantine/dead_a/` 이동 | Win32/GTK 검증 완료 | v0.0.31 종료 직후 |
| 3 | DEAD-B + Mutation → `quarantine/research/` 이동 | 동일 조건 | 동일 |
| 4 | quarantine 생존 확인 | 1~2주 운영 모니터링 | v0.0.32 초기 |
| 5 | 생존 실패 항목 최종 삭제 | 생존 확인 완료 | v0.0.32 중반 |

### 위험 관리

> **"지금은 기능 회귀가 더 무섭고, 레거시 제거는 언제든 할 수 있다."**
>
> — Win32/GTK 검증이 최우선. 실제 파일 이동/삭제는 검증 완료 후 단일 커밋으로 처리.

---

## A. Appendix: GEMINI.md Layer Map 대조표

| GEMINI.md 분류 | 본 문서 분류 | 항목 수 |
|----------------|-------------|---------|
| ACTIVE | 6. ACTIVE | 19 |
| REACHABLE | 7. REACHABLE | 2 |
| DORMANT | 5. DORMANT 전략 자산 | 15 |
| DEAD-A | 3. 즉시 제거 후보 | 6 |
| DEAD-B | 4. 연구 자산 (`quarantine/research/`) | 13 |
| LEGACY | 8. LEGACY | 2 |

> ⚠️ DEAD-B 13개 중 Semantic 계열 11개는 `research/`, `MutationCampaign`/`CampaignRunner` 2개도 `research/`. 모두 삭제가 아닌 격리.
