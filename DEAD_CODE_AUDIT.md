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

### 철학 전환의 실제: 변이는 완전히 사라진 것이 아니다

`CorpusExecutor::execute_shadow_campaign()`, `PredictiveImmuneLayer`, `TaxonomyMigrationManifest`, `Shadow Campaign`은 현재도 ACTIVE입니다.
즉 AXON은 변이(mutation)를 완전히 버린 것이 아니라 **자동 변이(Auto Mutation)만 제거**하고, **관찰용 변이(Observational Mutation)는 유지**한 상태입니다.

| 구분 | 예 | 상태 |
|------|-----|------|
| 자동 변이 | `MutationCampaign`, `EvolutionPipeline`, 자동 승인 | **제거** |
| 관찰용 변이 | `Shadow Campaign`, `PredictiveImmuneLayer`, Corpus 기반 회귀검증 | **ACTIVE 유지** |

이 차이는 DEAD-B 중 `MutationCampaign`/`CampaignRunner`를 단순 DEAD가 아닌 `FROZEN`으로 분류해야 하는 근거가 됩니다.

---

## 2. Classification Framework

GEMINI.md §Module State Taxonomy 기반 6가지 상태 + 사용자 분석 기반 2가지 처리 분류:

| 상태 | 의미 | 판단 기준 |
|------|------|-----------|
| **ACTIVE** | 실제 실행 경로에서 호출됨 | `execute_one_task()`에서 직접/간접 호출 확인 |
| **REACHABLE** | 현재 호출되지 않지만 파이프라인에 연결 가능 | mod.rs 선언 + builder/getter 패턴 |
| **DORMANT** | 현재 도달 불가능하지만 미래 기능으로 보존 가치 있음 | 완전 구현 + 대체재 없음 + v0.1.x 로드맵 연결 |
| **FROZEN** | DEAD는 아니나 현재 철학과 불일치, 보존 가치 있음 | 이전 철학 잔재 + v0.1.x 재활성화 가능성 높음 |
| **DEAD** | 호출 없음 + 의미 소멸 | 대체재 존재 |
| **LEGACY** | 과거 시스템 호환, 메인 파이프라인 미연결 | legacy 전용 경로에서만 호출 |
| **PASSIVE** | 실행되지만 관측만 수행, 흐름 미차단 | 호출됨 + 결과 무시 (WARN-only) |

### 처리 분류

| 분류 | 처리 | 조건 |
|------|------|------|
| **즉시 제거 후보** | `quarantine/dead_a/` 이동 | DEAD + 대체재 명확 |
| **FROZEN (재활성화 대기)** | `quarantine/research/` 이동 | FROZEN + v0.1.x에서 재활성화 예상 |
| **연구 자산** | `quarantine/research/` 이동 | DEAD + 연구 가치 있음 (Semantic 계열) |
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

## 4. FROZEN — 재활성화 대기 자산 → `quarantine/research/` 이동

`MutationEngine → ContractEngine` 전환 과정에서 자동 변이만 제거되고 관찰용 변이는 유지되었습니다.
`MutationCampaign`/`CampaignRunner`는 **DEAD가 아니라 FROZEN** — v0.1.x Corpus 기반 회귀검증 강화 시 유사 개념이 반드시 재등장합니다.

| 구분 | 상태 | 근거 |
|------|------|-------|
| 자동 변이 (EvolutionPipeline, on_validation_cycle) | **DEAD** | 철학 자체가 소멸 |
| 변이 실험장 (MutationCampaign, CampaignRunner) | **FROZEN** | 철학은 전환되었으나 Corpus/Shadow Campaign 강화 시 재활성화 가능성 높음 |

### 4.1 `MutationCampaign` / `CampaignRunner`

| 항목 | 값 |
|------|-----|
| **분류** | ❄️ **FROZEN** (DEAD 아님) |
| **위치** | `crates/axon-daemon/src/intelligence/mutation/` |
| **호출 여부** | ❌ |
| **대체재** | 없음 |
| **보존 사유** | AXON 초창기의 "변이 실험장" 계열. 향후 `CorpusExecutor::execute_shadow_campaign()`, Regression Corpus, Shadow Campaign을 강화할 계획이 있다면 다시 살아날 확률 높음. |
| **재활성화 조건** | v0.1.x Corpus 기반 회귀검증 강화, Shadow Campaign 고도화 |

**권장:** `quarantine/research/`로 이동. **삭제 금지.**

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

DORMANT 내에도 활성화 확실성에 따라 3단계로 구분합니다.

| 등급 | 의미 | 대상 |
|------|------|------|
| ⭐ **AXON 2세대 핵심** | AXON의 다음 진화 단계에서 반드시 필요 | `LowerInsertField` 계열, `StructuralRoundtripValidator` |
| 🟢 **전략 자산** | 특정 조건에서 활성화, 보존 가치 확실 | `SurgicalEditor` 계열, `CrashSandbox`, `TreeSitterLocator`, `AnchorValidator` |
| 🟡 **보존** | 가치는 있으나 활성화 조건이 불명확 | `StabilityMatrixHarness`, `RustCanonicalizer`, AST 검증기 3종, `dual_run_shadow_validation` |

### ⭐ 5.1 `LowerInsertField` / `LowerAppendStmt` / `LowerReplaceFnBody` — AXON 2세대 핵심

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT — ⭐ **AXON 2세대 핵심** |
| **가치** | 🔴 **최상위 — Diff IR의 전신** |
| **활성화 조건** | 파일 단위 수정 → 함수/블록/AST 단위 수정으로 진화할 때 |
| **예상 시점** | v0.1.x |

현재 AXON은 **파일 전체 수정** 위주입니다. 미래 AXON은 **함수 수정 → 블록 수정 → AST 단위 수정**으로 진화합니다. 이 세 개는 `Diff IR`의 전신입니다. 즉 단순한 DORMANT가 아니라 **AXON 2세대 아키텍처의 핵심 구성 요소**입니다.

```
지금:  Spec → File (파일 전체)
미래:  Spec → IR → Diff IR (함수/블록/AST 단위)
                           └── LowerInsertField
                           └── LowerAppendStmt
                           └── LowerReplaceFnBody
```

### ⭐ 5.2 `StructuralRoundtripValidator` — IR 시대의 필수 검증기

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT — ⭐ **AXON 2세대 핵심** |
| **가치** | 🔴 **거의 확실하게 미래에 필요** |
| **활성화 조건** | AXON이 `Spec → IR → Source → IR` 라운드트립 경로를 사용하게 될 때 |
| **예상 시점** | v0.1.x (IR 기반 수술 도입 시) |

현재는 `Spec → File`이지만 장기적으로는 `Spec → IR → Source → IR` 구조가 될 가능성이 높습니다. 그 순간 Roundtrip Validation은 필수입니다. 삭제 후보가 아니라 **잠자는 전략 자산**입니다.

### 🟢 5.3 `SurgicalEditor` / `SurgicalReplayHarness`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT — 🟢 전략 자산 |
| **가치** | 🟢 LowerInsertField 계열의 실행 파트너 |
| **활성화 조건** | AST 단위 정밀 수술 도입 시 |
| **예상 시점** | v0.1.x |

`LowerInsertField` 계열이 "무엇을 수정할지" 정의한다면, `SurgicalEditor`는 "어떻게 수정할지" 실행합니다. 쌍으로 부활할 가능성 높음.

### 🟢 5.4 `CrashSandbox`

| 항목 | 값 |
|------|-----|
| **분류** | DORMANT — 🟢 전략 자산 |
| **가치** | 🟢 런타임 검증 확장의 핵심 |
| **활성화 조건** | 런타임 물리 센서/샌드박스 실행 계열 확장 시 |
| **예상 시점** | v0.1.x |

런타임 검증을 샌드박스에서 격리 실행하는 개념. 현재 `execution_validator::validate()`가 단순 컴파일 검증만 수행하므로, 크래시/메모리 검증으로 확장 시 필요.

### 🟢 5.5 AST 검증기 3종

| 항목 | 값 |
|------|-----|
| **항목** | `AstOwnershipValidator`, `RegexAstValidator`, `TreeSitterAstValidator` |
| **분류** | DORMANT — 🟢 전략 자산 |
| **가치** | 🟢 AST 단위 수술의 전제 조건 |
| **활성화 조건** | AST 단위 수술 또는 Tree-sitter 기반 정적 분석 강화 시 |

### 🟡 5.6 기타 DORMANT

| 항목 | 등급 | 비고 |
|------|------|------|
| `StabilityMatrixHarness` | 🟡 보존 | 안정성 매트릭스 자동 측정 |
| `RustCanonicalizer` | 🟡 보존 | Semantic Canonicalizer의 상위 — 단, 하위 구조체는 연구 격리 |
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

## 6. ACTIVE 현황 (19개) — 가장 위험한 것은 DEAD가 아니라 ACTIVE

Pipeline Core 19개 항목. 유지.

### ⚠️ 위험 신호: `execute_one_task()` 집중 병목

DEAD보다 더 중요한 것은 ACTIVE 목록의 구조적 위험입니다.
ACTIVE 19개 중 거의 전부가 **pipeline.rs의 단일 함수 `execute_one_task()`** 에 몰려 있습니다.

```
모든 ACTIVE 경로
     ↓
execute_one_task()  ← 2000줄에 가까워지는 단일 함수
     ↓
junior → validator → lsp → compiler → senior → boss → promotion
```

**리스크:**
1. `execute_one_task()` 하나가 전체 파이프라인의 진입점이자 제어 흐름 — 책임 과중
2. 함수 길이가 2000줄에 근접 — 가독성 저하, 테스트 난이도 상승
3. 모든 게이트(GLOBAL_HARNESS, LSP, COMPILER, SENIOR)가 동일 함수 내에서 순차 실행 — 병목 지점 분리 불가
4. 단위 테스트 시 `execute_one_task()` 전체를 mock해야 함 — 테스트 피로도 높음

**권장 (v0.0.32 이후):**
- `execute_one_task()`에서 각 Gate를 별도 함수/모듈로 분리
- 각 Gate별 독립 테스트 가능하도록 인터페이스 추출
- Phase 1/2/3 실행 루프도 별도 함수로 분리 검토

### ACTIVE 목록

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
| FROZEN (Mutation 계열) | 2 | `quarantine/research/` 이동 (삭제 금지) | Win32/GTK 검증 완료 후 |
| 연구 자산 (Semantic 계열) | 11 | `quarantine/research/` 이동 | Win32/GTK 검증 완료 후 |
| DORMANT ⭐ 2세대 핵심 | 4 | 현재 위치 보존 (최우선) | v0.1.x 로드맵 연결 |
| DORMANT 🟢 전략 자산 | 4 | 현재 위치 보존 | v0.1.x 로드맵 연결 |
| DORMANT 🟡 보존 | 5 | 현재 위치 보존 | — |
| ACTIVE | 19 | 유지 (+ `execute_one_task()` 분산 리팩토링 검토) | v0.0.32 |
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

### ACTIVE 집중도 관리 (v0.0.32 과제)

현재 시점에서는 코드 삭제보다 **ACTIVE 파이프라인 집중도 분석**이 더 가치 있는 작업입니다.
`execute_one_task()`가 이미 2000줄 가까이 커졌다면, 다음 병목은 데드코드가 아니라 그 함수의 책임 분산입니다.

---

## A. Appendix: GEMINI.md Layer Map 대조표

| GEMINI.md 분류 | 본 문서 분류 | 항목 수 |
|----------------|-------------|---------|
| ACTIVE | 6. ACTIVE | 19 |
| REACHABLE | 7. REACHABLE | 2 |
| DORMANT | 5. DORMANT 전략 자산 | 15 |
| DEAD-A | 3. 즉시 제거 후보 | 6 |
| DEAD-B | 4.1 FROZEN (Mutation 계열) + 4.2 연구 자산 (Semantic 계열) | 13 |
| LEGACY | 8. LEGACY | 2 |

> ⚠️ GEMINI.md의 DEAD-B 13개는 본 문서에서 `FROZEN` 2개(MutationCampaign/CampaignRunner)와 `연구 자산` 11개(Semantic 계열)로 재분류됨. 모두 삭제가 아닌 `quarantine/research/` 격리.

## B. Appendix: Semantic 계열의 맥락 — 왜 함부로 삭제하면 안 되는가

### 문제: 현재 AXON이 직면한 근본적인 질문

최근 GTK / Win32 / Lua / Rust / C를 돌면서 결국 부딪힌 문제는 하나입니다:

> **"언어마다 다른 구조를 어떻게 공통적으로 표현할 것인가"**

Semantic 계열은 바로 이 질문에 대한 초창기 답변입니다.

### 각 항목의 미래 가치

| 항목 | 미래 질문 | 재활성화 조건 |
|------|-----------|---------------|
| `SemanticDistance` | "두 IR 노드가 의미적으로 얼마나 가까운가?" | IR diff / 머지 충돌 해결 |
| `CanonicalSemanticForm` | "다른 표현을 같은 의미로 정규화하려면?" | 다중 언어 공통 IR |
| `SignatureVector` | "함수 시그니처를 벡터로 표현하면?" | IR 기반 함수 탐색/매칭 |
| `RawAstNode` | "AST 노드를 IR과 연결하려면?" | AST 단위 수술 |
| `VisibilityScope` | "심볼의 가시 범위를 IR에 매핑하려면?" | IR 레벨 스코프 분석 |
| `TopologyEdge` | "모듈 간 의존선을 IR에 표현하려면?" | IR 토폴로지 분석 |

### 결론

지금은 사용하지 않아도, IR 레벨 수술로 가면 이 이름들은 다시 등장할 확률이 높습니다.
따라서 `quarantine/research/` 이동이 정확한 판단입니다.
