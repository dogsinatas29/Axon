# 🚀 AXON v0.0.25 FINAL BOOTSTRAP HARNESS

---

# 🔒 Principles (NON-NEGOTIABLE)

LLM Coding Principles:

1. Think Before Coding
2. Simplicity First
3. Minimal Changes
4. Goal-Oriented Execution
5. No Hallucinated APIs
6. Stable Code Protection
7. Context Confirmation

❗ 위 원칙 위반 시 즉시 실패 처리

---

# 🧭 GLOBAL EXECUTION RULE

각 단계는 반드시 아래 순서를 따른다:

1. 문서 읽기
2. Goal 정의
3. 적용 범위 확인
4. 최소 변경 구현 (diff only)
5. 테스트 작성
6. 테스트 실행
7. 실패 시 롤백
8. 통과 시 다음 단계

---

# 🧱 STAGE 0 — PRE-CHECK

## Goal

현재 시스템이 테스트 가능한 상태인지 확인

## Check

* cargo build 가능
* 테스트 실행 가능
* axon daemon 정상 실행

## Pass Condition

* 컴파일 성공

---

# 🧩 STAGE 1 — REWRITE SAFETY (이미 완료됨)

## Goal

Destructive rewrite 차단

## Validation

TC1 / TC2 / TC3 모두 통과

## Pass Condition

* 부분 rewrite → REJECT
* 전체 rewrite → PASS

---

# 🧠 STAGE 2 — COORDINATOR 도입

## 📄 문서

→ v0.0.25_coordinator.md
→ v0.0.25_coordinator_diff.md

## Goal

멀티 워커에서도 deterministic execution 보장

## Implementation

* coordinator.rs 추가
* daemon loop 교체
* worker loop 수정

## Test

### TC1

동일 파일 2개 task
→ 동시에 실행 금지

### TC2

dependency 있는 task
→ 순서 보장

## Pass Condition

* race condition 없음
* file lock 정상 동작

---

# 🔒 STAGE 3 — SINGLE FILE SANDBOX

## Goal

오직 target_file만 수정 가능

## Implementation

* write gate 추가
* target_file 외 변경 → 즉시 실패

## Test

### TC1

다른 파일 수정 시도
→ FAIL

### TC2

target_file만 수정
→ PASS

---

# ⚙️ STAGE 4 — STATIC VALIDATOR

## 📄 문서

→ v0.0.25_execution_final_gate.md

## Goal

실행 이전에 쓰레기 코드 차단

## Rules

* F_STUB
* F_MARKDOWN
* F_HARDCODE

## Test

### TC1

```rust
// empty
```

→ F_STUB

### TC2

````rust
```rust
````

````
→ F_MARKDOWN

### TC3
2023 포함
→ F_HARDCODE

---

# 🧪 STAGE 5 — EXECUTION VALIDATOR

## Goal
“실제로 돌아가는 코드만 통과”

## Implementation
- compile_check
- run_check

## Test

### TC1
컴파일 실패 코드
→ F_COMPILE_FAIL

### TC2
panic 코드
→ F_RUNTIME_FAIL

### TC3
정상 코드
→ PASS

---

# 🧠 STAGE 6 — SENIOR GATE HARDENING

## 📄 문서
→ v0.0.25_senior_gate_hardening.md

## Goal
False APPROVE 0%

## Implementation

❌ 기존:
```rust
contains("APPROVE")
````

✅ 변경:

* [APPROVE] / [REJECT] strict parsing

## Test

### TC1

"REJECT ... APPROVE"
→ FAIL

### TC2

[APPROVE]
→ PASS

### TC3

no tag
→ FAIL

---

# 🧠 STAGE 7 — JSON SENIOR GATE

## Goal

문자열 기반 제거 → 완전 구조화

## Output Format

```json
{
  "decision": "APPROVE",
  "reason": "..."
}
```

## Validation

### TC1

invalid JSON → FAIL

### TC2

decision 없음 → FAIL

### TC3

APPROVE → PASS

---

# 🔥 STAGE 8 — FINAL GATE 통합

## 📄 문서

→ v0.0.25_execution_final_gate.md

## Pipeline

```
Coordinator
→ Sandbox
→ Static Validator
→ Execution Validator
→ Senior Gate
→ Atomic Commit
```

## Test

### TC1

stub → FAIL

### TC2

compile fail → FAIL

### TC3

runtime fail → FAIL

### TC4

senior reject → FAIL

### TC5

all pass → COMMIT

---

# 💣 STAGE 9 — ATOMIC COMMIT 검증

## Goal

실패 시 디스크 오염 0

## Test

### TC1

execution 실패
→ 파일 unchanged

### TC2

성공
→ 파일 교체

---

# 🧪 STAGE 10 — FULL PIPELINE TEST

## 시나리오

1. input.rs → 정상
2. calculation.rs → compile fail
3. dependencies.rs → stub

## 기대 결과

* calculation.rs → FAIL
* dependencies.rs → FAIL
* input.rs → PASS
* 전체 pipeline STOP

---

# 🏁 FINAL ACCEPTANCE CRITERIA

모두 만족해야 완료:

* stub 0%
* markdown 0%
* compile fail 0%
* runtime fail 0%
* false approve 0%
* race condition 0%
* rollback 100% 보장

---

# 🔚 결론

이 부트스트랩은 단순 실행 문서가 아니다.

👉 **“LLM을 통제 가능한 컴파일러로 강제하는 절차”**

여기까지 통과하면:

* 오지랖 → 구조적으로 불가능
* 스터브 → 통과 불가능
* 잘못된 승인 → 불가능
* 깨진 코드 → 생성 불가능

---

# STATUS

```
READY FOR EXECUTION
```

