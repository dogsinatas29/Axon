# AXON Changelog

## v0.0.31 - Transport Layer Hardening & Phase Gating
- **Phase Gating (Phase 7-D)**: Fixed `resolve_target()` closure in pipeline Phase filters to parse `target_file` from `task.title` when `None`, ensuring correct file targeting across all phases.
- **Transaction Envelope (Phase 8-A/B/C)**: Added `PatchEnvelope` struct with fields (`patch_id`, `target`, `body`, `is_complete`, `integrity_errors`, `validate()`). Junior agents now wrap patches in `===AXON_PATCH_BEGIN===` → `BODY` → `===AXON_PATCH_END===` format.
- **Envelope Parser (Phase 8-C)**: Implemented `extract_patch_envelope()` with structural integrity checks (BEGIN/BODY/END presence). Optional `BYTE_COUNT` and `CHECKSUM` validation (only enforced if declared).
- **Causal Rejection (Phase 8-D)**: Updated `normalize_senior_output()` to detect empty responses (`{}`) as `PATCH_TRUNCATED` hard reject. Senior agents provide structured JSON feedback for precise self-correction.
- **Empty Validator = Unsafe**: Senior returning `{}` or empty response is treated as hard reject, not neutral. This prevents silent failure propagation from large-file context collapse, silent truncation, and fake success.

## v0.0.30 - Governance Hardening & Production Integrity (FINAL)
- **BossBoard v2 (The Command Center)**: Implemented a grid-locked viewport with a fixed tactical command desk, ensuring the Boss always has immediate access to [SEAL/REWORK] actions.
- **Sacred Contract Portal**: Added a glass-morphism modal for real-time viewing of the full Architectural IR (Sacred Contract), providing absolute legal grounding for every decision.
- **Violation Trace UI**: Introduced a high-density violation analysis sidebar to pinpoint specific symbol mismatches and physical errors (e.g., SQLite3 protocol violations).
- **Zero-Warning Production Engine**: Fully reconstructed the `TEST2/spec` project to achieve **100% build success (0 warnings)** with gcc 15.2.0, enforcing strict SQLite3 C API lifecycle compliance.
- **Strategic Phase Sorting**: Reordered all task and risk lists to follow the logical manufacturing sequence: **Phase 1 (Headers) → Phase 2 (Implementations) → Phase 3 (Integration)**.
- **Daemon Hardening**: Eliminated all internal Rust compiler warnings to ensure 100% engine purity and reliability.
- **C-Native Focus**: Current IR hardening and production validation in v0.0.30 have been focused exclusively on the C language stack.

## v0.0.29 - Deterministic Synthesis Pipeline & Semantic Hardening
- **Semantic Stability Hardening**: Implemented `SpecAnalysis` stage to extract `ImmutableConstraints`, preventing Architect agents from arbitrarily upgrading 'Optional' components to 'Core' status.
- **Task Graph & Ordering Validation**: Enforced strict dependency-aware ordering (Headers → Implementations → main) and verified task counts against the IR.
- **Header Freeze Protocol**: Mandatory declaration-only enforcement for C header files to prevent logic leakage into interfaces.
- **Fail-Fast & Rework Limit**: Integrated a global watchdog that aborts the pipeline after 3 failed rework attempts to prevent infinite token loops and ensure convergence.
- **🚨 Critical Flaw & Audit Failure Analysis (v0.0.29.25)**:
  - **Ghost Structs**: Identified missing field definitions in `architecture.md` leading to hallucinated data models (e.g., `struct user_record` vs `struct user`).
  - **Senior Audit Failure**: Identified "Blind Approvals" where Senior agents used incorrect feedback logic (e.g., approving non-DB modules using SQLite3 validation logs).
  - **Interface Drift**: Detected severe semantic mismatch between IR signatures and actual implementation, risking linker failures.
  - **Logical Over-complication**: Observed Junior agents violating KISS principles, leading to self-trapped logical loops during the 90+ GCC rejection cycles.

## v0.0.28 - Architectural Auditor & Contract Consistency
- **Architectural Auditor**: Refactored the Senior Agent into a strict binary compliance auditor (Contract Verifier), eliminating subjective style rejections and focusing purely on SSOT integrity.
- **Structural Hardening**: Added Anti-Stub v3 and Header Guard to prevent placeholder logic and enforce source/header separation.
- **Intelligent Fault Localization (Self-Healing)**: Implemented a compiler-aware diagnosis layer that pinpointed exact source files (e.g., `main.c`) responsible for build failures, enabling surgical reworks and preventing infinite loops.
- **Self-Healing Reviewer**: Implemented a 3-attempt retry loop for JSON protocol compliance, preventing pipeline deadlocks caused by small model formatting errors.
- **Deterministic Materializer**: Hardened Junior Agent prompts to enforce absolute IR contract faithfulness (Consistency > Quality), suppressing function renaming and hallucinated headers.
- **Header Resolver (v2)**: Fixed C-header mapping bug to support decoupled source/include structures (`src/*.c` <-> `include/*.h`) in the integrity gate.
- **Task ID Namespace**: Implemented `hdr_` and `impl_` prefixing for deterministic task/thread isolation in the Work Board.
- **Traceable Reworks**: Added `(Rework #N)` tagging to task titles for clear generation tracking and UI observability.
- **언어별 IR 분리 후 C 언어 테스트**: 언어별 독립 IR 아키텍처 적용 후 C 언어 환경에서 파이프라인 정합성 최종 검증 완료.

## v0.0.25 - The Universal Factory (Phase 0-8)
- **Language-Agnostic Engine**: Full native support for **C, Rust, and Python** projects with automatic entry point detection.
- **Lounge (Vibe) System**: Real-time broadcasting of agent thoughts to the `#lounge` channel without polluting source files.
- **Atomic Write Hardening**: Guaranteed code integrity through function-signature preservation and mandatory physical validation gates.
- **Studio UI Isolation**: Strictly separated work boards and global system channels for a focused development experience.
- **Scope Control (Phase 2)**: Real-time detection and rejection of forbidden patterns to ensure architectural compliance.

## v0.0.24 - Factory Pipeline Hardening & i18n
- **Parallel Race Condition Fix**: Refactored `STE_SHIELD` and `WRITE_GATE` to validate against 'Initial Simulated State' instead of physical disk, enabling stable multi-worker execution.
- **Full i18n Stack**: Global support for KR/EN/JP across CLI and Studio UI with production build optimization.
- **Lounge Activation**: Real-time EventBus broadcasting for agent "Nogari" (chatter). Dynamic inner thoughts are now visible on the dashboard.
- **Approval Flexibility**: Senior/Architect gates now accept Markdown bolding (`**APPROVE**`) for better small-model compatibility.
- **Connectivity Guard**: Real-time validation of LLM endpoints during initialization to prevent silent connection failures.
- **Audit Preservation**: Mandatory DB logging for all failed proposals in the `posts` table for easier debugging.

## v0.0.23 - Maximum Harness & Physical Hardening
- **Maximum Harness Protocol**: Integrated physical signature matching (F8.1) and high-density logic verification gates.
- **Pollution Shield**: Automatic detection and rejection of markdown code blocks (triple backticks) to ensure code purity.
- **COMMIT_PENDING Pipeline**: Split into Logical Approval → Materialization → Physical Validation.
- **Agent Signature Persistence**: Guaranteed embedding of Agent ID/Task ID for 100% traceability.
- **S.T.E. Shield & WIPE_SHIELD**: Hardened security against unauthorized drifts and low-quality stubs (Anti-Stub v2).
- **Auto-Rollback & Requeue**: Immediate revert on failure with persistent self-correction loop.

## v0.0.22 - Deterministic Factory Pipeline
- **IR Convergence Loop**: Auto-repair loop until fixed-point IR is reached.
- **Stage 3.5 Stub Generation**: Proactive skeleton code generation for dependency resolution.

## v0.0.18 - Bug Arrest & Quota Management
- **3-Tier Parser**: Guaranteed code extraction even from corrupted LLM outputs.
- **0-byte Bug Fix**: Resolved daemon merge logic failures.
- **503 Mitigation**: Added wait logic for Gemini API quotas.

## v0.0.17 - Control & Isolation
- **Multi-Agent Orchestration**: JNR -> SNR -> ARCH command hierarchy.
- **Ollama Adapter**: Local model execution and performance tracking integration.
