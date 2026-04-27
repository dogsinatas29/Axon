# 📄 PHASE_03_BOOTSTRAPPING.md (한글 버전 v2 - AvailableAgents 추가)

## 🎯 목적
검증된 설정과 상태를 기반으로 **실행 컨텍스트를 구성**한다.

---

## 1. 입력

- CONFIG LOCKED (PHASE_01 출력)
- ValidationResult (PHASE_02 출력)

---

## 2. 처리 규칙

- CONFIG는 절대 변경 금지
- Validation 결과는 그대로 반영
- 경고(WARN/FAIL)는 반드시 포함
- 누락된 정보 생성 금지

---

## 3. 컨텍스트 구성

### Agents (전체)
- role
- model
- runtime

### AvailableAgents (실행 가능 - OK only)
- role:
  - id
  - status: OK

### Constraints
- queue_limit
- sampling_rate

### Warnings
- agent_id:
  - role
  - status: WARN | FAIL
  - reason

---

## 4. 출력 포맷

[CONTEXT]

Agents:
- role:
- model:
- runtime:

AvailableAgents:
- role:
  - id:
  - status: OK

Constraints:
- queue_limit:
- sampling_rate:

Warnings:
- agent_id:
  - role:
  - status:
  - reason:

---

## 5. Assertion (하네스 필수)

- 실행 단계에 전달되어야 하는 핵심 정보는 무엇인가?
- 왜 AvailableAgents가 필요한가?
- Validation 결과는 어떻게 분리되는가?

---

## 🔒 잠금 조건

- 모든 Agent 포함
- AvailableAgents 명확히 분리됨
- Constraints 포함
- Warnings 반영됨

---

## 🚫 강제 제약 조건

- 시스템은 새로운 Agent를 생성하지 않는다
- 시스템은 기존 Agent를 제거하지 않는다
- 시스템은 Validation 결과를 무시하지 않는다

---

## 🧠 출력 계약

ExecutionContext:

- Agents
- AvailableAgents
- Constraints
- Warnings
