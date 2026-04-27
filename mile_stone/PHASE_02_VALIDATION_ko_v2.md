# 📄 PHASE_02_VALIDATION.md (한글 버전 v2 - role 포함)

## 🎯 목적
모든 에이전트가 실제 실행 가능한 상태인지 검증하고,  
결과를 명시적으로 기록한다.

---

## 1. 입력

- CONFIG LOCKED (PHASE_01 출력)

---

## 2. 처리 규칙

- 모든 에이전트는 반드시 검증 대상
- 검증 생략 금지
- 결과 없이 다음 단계 진행 금지
- 시스템은 에이전트를 수정하거나 교체하지 않는다

---

## 3. 검증 기준

### Local Runtime
- endpoint 접근 가능 여부
- 모델 존재 여부 확인

### Cloud Runtime
- API Key 존재 여부
- 기본 요청 성공 여부

---

## 4. 결과 포맷

[VALIDATION RESULT]

Agents:
- id:
  - role:
  - status: OK | WARN | FAIL
  - reason:

---

## 5. 상태 정의

- OK: 정상 작동 가능
- WARN: 실행 가능하지만 문제 존재
- FAIL: 실행 불가

---

## 6. Assertion (하네스 필수)

- FAIL 발생 시 어떻게 처리되는가?
- 시스템이 에이전트를 자동으로 교체하는가?
- 이 단계의 출력은 다음 단계에서 어떻게 사용되는가?

---

## 🔒 잠금 조건

- 모든 에이전트에 대해 role + status 존재
- 상태가 OK / WARN / FAIL 중 하나로 정의됨
- reason이 명시됨

---

## 🚫 강제 제약 조건

- 시스템은 에이전트를 자동 교체하지 않는다
- 시스템은 모델을 변경하지 않는다
- 시스템은 역할을 수정하지 않는다

---

## 🧠 출력 계약

ValidationResult:

- Agents:
  - id
  - role
  - status
  - reason
