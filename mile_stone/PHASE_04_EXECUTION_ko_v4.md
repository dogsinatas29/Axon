# 📄 PHASE_04_EXECUTION.md (한글 버전 v4 - Hard Constraint + Runtime Failure 분리)

## 🎯 목적
구성된 실행 컨텍스트를 기반으로 **정해진 파이프라인을 따라 작업을 처리**하며,  
정적 실패(STATIC_FAIL)와 실행 중 실패(RUNTIME_FAIL)를 명확히 구분한다.

---

## 1. 입력

- ExecutionContext (PHASE_03 출력)
- ValidationResult (STATIC_FAIL 포함)

---

## 2. 처리 규칙

- 역할 순서 절대 변경 금지
- 모델 자동 선택 금지
- 역할 재할당 금지
- Validation 상태(STATIC_FAIL)를 반드시 반영

---

## 3. 실행 흐름

Task
 ↓
Junior
 ↓
Senior
 ↓
Architect
 ↓
Commit

---

## 4. 에이전트 선택

- 동일 역할 내에서 round-robin 방식 사용
- 임의 선택 금지
- 자동 최적화 금지

---

## 5. 🔒 최소 레이어 강제 조건 (Hard Constraint)

다음 조건을 반드시 만족해야 실행 가능:

- Junior layer: 최소 1개 이상의 OK 상태 agent 존재
- Senior layer: 최소 1개 이상의 OK 상태 agent 존재

조건 불만족 시:

[EXECUTION BLOCKED]

Reason:
- Minimum layer requirement not satisfied

---

## 6. FAIL 유형 정의

### STATIC_FAIL (Validation 단계)

- 실행 전 이미 불가능한 상태
- 예: endpoint 없음, model 없음, 인증 실패

처리:
- execution에서 무조건 skip

---

### RUNTIME_FAIL (Execution 단계)

- 실행 중 발생하는 실패
- 예: timeout, rate limit, crash

---

## 7. 🔁 RUNTIME FAILURE 정책

- 실패 발생 시 retry 수행 (max 3)
- retry 실패 시 해당 agent skip
- skip 이후에도 최소 레이어 조건 재검증

재검증 결과:

- 조건 유지 → 계속 실행
- 조건 붕괴 → 즉시 EXECUTION BLOCKED

---

## 8. 제어 메커니즘

### Queue Limit

- queue 길이가 limit 초과 시:
  → Junior dispatch 중단

---

### Sampling Review

- 확률 기반 검토 수행
- sampling_rate 사용

---

### Retry 정책

- max retry = 3
- 초과 시 Architect로 escalation

---

## 9. Assertion (하네스 필수)

- STATIC_FAIL과 RUNTIME_FAIL의 차이는 무엇인가?
- 최소 레이어 조건은 언제 재검증되는가?
- 실행이 중단되는 정확한 조건은 무엇인가?

---

## 🔒 잠금 조건 (Lock Condition)

다음 조건을 모두 만족할 때까지 절대 다음 단계로 진행 금지:

- 최소 레이어 조건 검증 완료
- FAIL 유형이 명확히 구분됨
- 실행 또는 차단 결과가 명시됨

---

## 🚫 강제 제약 조건

- 시스템은 역할을 변경하지 않는다
- 시스템은 모델을 변경하지 않는다
- 시스템은 실행 흐름을 변경하지 않는다

---

## 🧠 출력 계약

ExecutionResult:

- status: RUNNING | BLOCKED
- reason (if blocked)
- Task 결과 (if running)
- 처리 경로
- Retry 횟수
- Skip 정보
- Failure type (STATIC / RUNTIME)

---

## 🧠 요약

이 단계는 다음을 수행한다:

정해진 파이프라인 → 구조 검증 → 실패 유형 분리 → 통제된 실행

시스템은 실행 가능성과 실행 중 실패를 구분하여 안정성을 유지한다.
