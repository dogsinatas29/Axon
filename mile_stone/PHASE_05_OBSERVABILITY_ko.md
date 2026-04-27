# 📄 PHASE_05_OBSERVABILITY.md (한글 버전)

## 🎯 목적
시스템의 실행 상태를 **실시간으로 측정하고 가시화**한다.

---

## 1. 입력

- ExecutionResult (PHASE_04 출력)

---

## 2. 처리 규칙

- 모든 실행 결과는 관측 대상
- 데이터 누락 금지
- 상태는 객관적으로 기록
- 해석/추론 금지 (측정만 수행)

---

## 3. 수집 항목

### Agents

- latency (응답 시간)
- status (OK / WARN / FAIL)

---

### Queue

- length (현재 대기열 길이)

---

### Failures

- count (실패 횟수)

---

## 4. 출력 포맷

[OBSERVABILITY]

Agents:
- id:
  - latency:
  - status:

Queue:
- length:

Failures:
- count:

---

## 5. Assertion (하네스 필수)

- 왜 관측 데이터는 해석이 아닌 기록이어야 하는가?
- 이 정보는 누가 사용하며, 어떤 결정을 위해 필요한가?
- Observability가 없을 경우 어떤 문제가 발생하는가?

---

## 🔒 잠금 조건 (Lock Condition)

다음 조건을 모두 만족할 때까지 종료 금지:

- 모든 Agent 상태가 기록됨
- Queue 상태가 기록됨
- Failure 데이터가 존재함

---

## 🚫 강제 제약 조건

- 시스템은 데이터를 수정하지 않는다
- 시스템은 데이터를 숨기지 않는다
- 시스템은 결과를 왜곡하지 않는다

---

## 🧠 출력 계약

ObservabilityReport:

- Agents:
  - id
  - latency
  - status

- Queue:
  - length

- Failures:
  - count

---

## 🧠 요약

이 단계는 다음을 수행한다:

실행 상태 → 측정 및 가시화

이 데이터는 사용자 및 상위 제어 시스템이  
병목, 실패, 성능을 판단하는 근거가 된다.
