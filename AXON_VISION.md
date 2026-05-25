# 🚀 AXON VISION & MANIFESTO
**"Deterministic Software Repair Kernel"**

현재 AXON의 핵심 목표는 "명세 → 코드 자동 생성"이 아닙니다.
현재 목표는 **"거대한 레거시 코드베이스를 결정론적으로 이해하고, 국소 수리(local repair)를 안전하게 수행하는 거버넌스 커널"**입니다. 특히 Rust, C, Win32 C를 중심으로 현실 세계의 Hostile Legacy 환경을 통제 가능한 형태로 수렴시키는 것이 핵심 방향입니다.

## 0. AXON의 4대 계층 본질

### A. Context OS (외부 기억 계층)
- **자산**: `GEMINI.md`, `milestone/*`, `context/*.synapse_gate.txt`, `ownership_snapshot.json`, `replay logs`
- **목적**: LLM과 인간 모두의 기억 붕괴(Context Evaporation) 방지

### B. Governance Kernel (결정권 관리)
- **자산**: Seal, Reopen, Ownership, Circuit Breaker, Failure Budget, Promotion Gate
- **목적**: AI의 무단 구조 변이 차단 및 인간 중심의 통제력 유지

### C. Deterministic Repair Engine (결정론적 수리 엔진)
- **자산**: Topology-aware repair, Semantic authority gate, Surgical edit, Replay determinism
- **목적**: 동일 실패 → 동일 복구 보장

### D. Corpus Immune System (코퍼스 면역 시스템)
- **자산**: Entropy corpus, Catastrophe archive, Replay campaign, Drift fingerprint
- **목적**: 현실 세계 데이터 실측을 통해 무엇이 위험한가를 통계적으로 기억하고 회피

---

## 1. 언어별 전략

### 🦀 Rust 전략 (가장 위험한 환경)
- **위험 요인**: Macro entropy, Trait topology, Generic explosion, Impl leakage, Formatter normalization drift
- **목표**: "Rust 전체 지원"이 아닌 **SAFE_SUBSET 기반 결정론적 수리**
- **핵심 통제**:
  1. **Skeleton Constitution 강제**: Phase1에서는 trait 선언, struct 선언, fn signature, module declaration만 허용 (impl body, helper abstraction, async runtime wiring 절대 금지).
  2. **Macro Quarantine**: 매크로는 `SHADOW_ONLY`로 격리하며, Authoritative path로 절대 승격시키지 않음.
  3. **Trait Topology Protection**: Signature hash, topology hash, ownership freeze로 ripple containment 유지.
  4. **Surgical Edit Only**: Printer-based rewrite 전면 금지. 오직 Byte surgery, anchor validation, stable edit plan 기반으로만 수정.

### ⚙️ C 전략 (AXON의 핵심 실험장)
- **특징**: 물리 구조가 단순, topology 추적 명확, header/source separation 존재, deterministic build 용이
- **목표**: 거대한 레거시 C 프로젝트를 부분 수정 가능한 상태로 만드는 것
- **핵심 통제**:
  1. **Header Constitution**: `.h` 파일은 declaration only (logic leakage 금지).
  2. **Ownership-based Compilation**: 함수, struct, typedef 단위의 심볼 단위 ownership 유지.
  3. **Ripple Containment**: 컴파일 에러 발생 시 file-wide reopen 금지, symbol-local repair 우선.
  4. **Legacy Corpus Expansion**: sqlite, gtk, curl, old game engines, embedded libraries 타겟.

### 🪟 Win32 C 전략 (OS ABI Governance)
- **특징**: 단순 C가 아닌 OS ABI 차원의 문제
- **목표**: Win32 GUI topology를 붕괴시키지 않는 local repair
- **핵심 통제**:
  1. **Fake Win32 금지**: fake HWND, fake message loop, fake CreateWindow, SDL masquerading 절대 금지.
  2. **Message Loop Integrity**: `GetMessage`, `TranslateMessage`, `DispatchMessage` 루프를 sacred topology로 보호.
  3. **WM_PAINT Locality Protection**: `BeginPaint`, `EndPaint` boundary drift 절대 금지.
  4. **PE Binary Validation**: 빌드 성공 여부에 의존하지 않고 subsystem, linkage, dll dependency 실측 검증.
  5. **Callback Ownership**: `WndProc` 및 callback chain 보호.

---

## 2. 현재 최우선 목표

- **[최우선]** 
  - A. Rust Skeleton Constitution 완성 (Phase 1 뼈대 강제)
  - B. Deterministic Replay 안정화
  - C. Phase-level replay tests
- **[중기]** 
  - D. GTK hostile corpus
  - E. Win32 hostile corpus
  - F. Legacy mutation campaigns
- **[장기]** 
  - G. Canonical Semantic Form 안정화
  - H. Promotion governance 자동화
  - I. Catastrophe immune memory 고도화

---

## 3. 앞으로의 POC 방향

- **POC 1 — Local Repair**: 전체 재생성 없이 특정 함수만 안전하게 수정
- **POC 2 — Deterministic Replay**: 동일 입력 → 동일 복구 경로 증명
- **POC 3 — Catastrophe Mapping**: 어떤 코드가 위험한가를 Heatmap으로 시각화
- **POC 4 — Legacy Immune System**: 과거 붕괴 패턴 기억 → 미래 붕괴 선제 차단

---

> **FINAL VISION**  
> AXON은 Copilot, Cursor, Agent IDE 계열과는 완전히 다른 시스템입니다.  
> 핵심 경쟁력은 "더 좋은 생성(Generation)"이 아니라,  
> **"LLM이 거대한 레거시 코드를 함부로 망치지 못하게 물리적으로 통제하는 운영체제"**가 되는 것에 있습니다.  
> 오직 **Replayability, Locality Preservation, Governance, Catastrophe Immunity, Externalized Cognition**만이 우리의 무기입니다.
