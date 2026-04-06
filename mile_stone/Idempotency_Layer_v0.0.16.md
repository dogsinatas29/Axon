# Idempotency Layer v0.0.16

## 1. 목적

동일 입력과 상태에서 동일 결과를 보장하고, 중복 실행 및 부작용을
방지한다.

------------------------------------------------------------------------

## 2. 핵심 정의

same input + same state -\> same output + no duplicated side effects

------------------------------------------------------------------------

## 3. Execution Fingerprint

fingerprint = hash(task_id + normalized_input + relevant_state_subset)

------------------------------------------------------------------------

## 4. 실행 전 체크

if fingerprint exists: skip execution reuse cached output

------------------------------------------------------------------------

## 5. Output Cache 구조

.execution_cache/ ├── `<fingerprint>`{=html}.json

내용: - task_id - fingerprint - output - timestamp - status

------------------------------------------------------------------------

## 6. Side Effect 통제

-   write-once or replace-only
-   append 금지
-   외부 API 중복 호출 금지

------------------------------------------------------------------------

## 7. Task 설계 규칙

-   output = f(input, state_subset)
-   deterministic
-   side-effect isolation

------------------------------------------------------------------------

## 8. Snapshot 연계

cache hit → 실행 생략 cache miss → 실행 후 cache 저장 + snapshot 생성

------------------------------------------------------------------------

## 9. 실패 시 동작

retry 시 fingerprint 동일 → cache 사용 → 재실행 방지

------------------------------------------------------------------------

## 10. 금지 사항

-   전체 state hash 사용 금지
-   timestamp 포함 금지
-   input normalization 누락 금지

------------------------------------------------------------------------

## 11. 결론

Idempotency는 최적화가 아니라 필수 안정성 계층이다.
