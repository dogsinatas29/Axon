# Runtime Integration Layer v0.0.16

## 1. 목적

전체 시스템(State / Scheduler / Executor / Guard / Snapshot /
Idempotency / Observability)을 하나의 실행 흐름으로 통합한다.

------------------------------------------------------------------------

## 2. 전체 Runtime Loop

    while not done:

      task = scheduler.next()

      guard.pre(task, state)

      result = executor.run(task, state)

      guard.post(state, result.state)

      snapshot.save(result)

      observability.log(task, result)

      state = result.state

------------------------------------------------------------------------

## 3. 컴포넌트 계약

### 3.1 Scheduler → Executor

입력: - task_id

출력: - 없음 (task 전달)

------------------------------------------------------------------------

### 3.2 Executor → Guard

Pre-check 입력: - task - state

Post-check 입력: - state_before - state_after

출력: - allow / reject

------------------------------------------------------------------------

### 3.3 Executor → Snapshot

입력: - task_id - state_after - output

출력: - snapshot_id

------------------------------------------------------------------------

### 3.4 Executor → Idempotency

입력: - task + state_subset

출력: - cache_hit / execute

------------------------------------------------------------------------

### 3.5 Executor → Observability

입력: - task - result - execution metadata

------------------------------------------------------------------------

## 4. State 관리 규칙

-   State는 Executor만 변경 가능
-   다른 레이어는 read-only
-   모든 변경은 write scope 기반

------------------------------------------------------------------------

## 5. Fail-Fast 정책

-   Guard 실패 → 즉시 중단
-   Validation 실패 → 즉시 중단
-   Contract 불일치 → 즉시 중단

------------------------------------------------------------------------

## 6. Loop Control

-   Scheduler가 종료 결정
-   더 이상 실행할 task가 없으면 종료

------------------------------------------------------------------------

## 7. 실패 처리 전략

-   retry: 동일 fingerprint → cache 사용
-   구조 위반: 즉시 abort
-   부분 실패: snapshot 기반 재실행

------------------------------------------------------------------------

## 8. 확장 포인트

-   병렬 실행 (독립 task)
-   distributed executor
-   snapshot diff 저장

------------------------------------------------------------------------

## 9. 금지 사항

-   state 직접 수정 (Executor 외)
-   validation 없는 실행
-   dynamic contract 변경

------------------------------------------------------------------------

## 10. 결론

Runtime Integration은 모든 레이어를 하나의 실행 흐름으로 묶는 마지막
단계다.

이 구조가 완성되면: - 시스템은 안정적으로 반복 실행 가능 - 실패에도 복구
가능 - LLM 제어 가능 상태 유지
