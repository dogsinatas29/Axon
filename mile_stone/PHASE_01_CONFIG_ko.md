# 📄 PHASE_01_CONFIG.md (한글 버전)

## 🎯 목적
사용자가 정의한 설정을 **변경 불가능한 실행 계약(Immutable Execution Contract)**으로 고정한다.

---

## 1. 입력

axon_config.json

---

## 2. 처리 규칙

- 값 수정 금지
- 기본값 자동 보완 금지
- 역할 재할당 금지
- 구조적 정규화만 허용

---

## 3. Agent Registry 구성

모든 에이전트를 명시적으로 정의해야 한다:

- Architect (정확히 1개)
- Seniors (0개 이상)
- Juniors (0개 이상)

각 에이전트는 반드시 포함해야 한다:
- role
- model
- runtime
- endpoint 또는 provider

---

## 4. 실행 설정

다음 항목을 확정한다:

- review_queue_limit
- sampling_rate
- fallback_enabled

---

## 5. 출력 계약 (Output Contract)

[CONFIG LOCKED]

Agents:
- id:
- role:
- model:
- runtime:

Execution:
- queue_limit:
- sampling_rate:
- fallback_enabled:

Invariant:
- 역할 변경 금지
- 모델 변경 금지
- 런타임 변경 금지
- 암묵적 기본값 금지

---

## 6. Assertion (하네스 필수)

- 이 단계의 역할은 무엇인가?
- Validation 단계로 무엇이 전달되는가?
- 이 단계 이후 무엇이 절대 변경 불가인가?

---

## 🔒 잠금 조건 (Lock Condition)

다음 조건을 모두 만족할 때까지 절대 다음 단계로 진행 금지:

- 모든 에이전트가 명시적으로 정의됨
- 모든 필수 필드가 존재함
- Output Contract가 완전히 생성됨

---

## 🚫 강제 제약 조건

- 시스템은 사용자 설정을 수정하면 안 된다
- 시스템은 누락된 값을 추론하면 안 된다
- 시스템은 역할을 재배치하면 안 된다

---

## 🧠 요약

사용자 의도 → 변경 불가능한 실행 계약

이 이후의 모든 단계는  
이 고정된 상태를 기반으로만 동작해야 한다.
