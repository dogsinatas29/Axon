# Recovery & Snapshot Layer v0.0.16

## 1. 목적

LLM 실행 중 실패 발생 시 전체 재실행 없이 마지막 정상 상태에서 복구
가능하도록 한다.

------------------------------------------------------------------------

## 2. 핵심 원칙

### 2.1 Task Boundary Commit

모든 Snapshot은 Task 단위로만 생성된다.

### 2.2 Immutable Snapshot

Snapshot은 생성 이후 절대 수정되지 않는다.

### 2.3 Snapshot = Source of Truth

Runtime state는 캐시이며 실제 기준은 Snapshot이다.

------------------------------------------------------------------------

## 3. Snapshot 정의

-   state
-   task_id
-   input / output
-   execution metadata (hash, duration, status)

------------------------------------------------------------------------

## 4. Snapshot 생성 규칙

Task 성공 시에만 snapshot 생성

------------------------------------------------------------------------

## 5. Recovery 흐름

Task 실패 → 마지막 Snapshot으로 rollback → 재실행

------------------------------------------------------------------------

## 6. State 포함 기준

포함: - execution 결과 - DAG 상태 - scheduler 상태

제외: - 로그 - UI 상태 - 임시 캐시

------------------------------------------------------------------------

## 7. 저장 구조

.snapshots/ ├── task_001.json ├── task_002.json

append-only 구조

------------------------------------------------------------------------

## 8. Idempotency 연계

execution_hash = hash(input + state_subset)

동일 hash 존재 시 재실행 생략

------------------------------------------------------------------------

## 9. 금지 사항

-   Task 중간 snapshot
-   mutable snapshot
-   partial state 저장
-   snapshot 없는 retry

------------------------------------------------------------------------

## 10. 결론

Task → Snapshot → Failure → Restore → Resume

이 루프가 유지되면 시스템은 안정적으로 복구 가능하다.
