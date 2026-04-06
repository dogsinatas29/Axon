# Executor Layer v0.0.16

## 1. 목적

Task를 실제로 실행하고 State를 변경하는 유일한 실행 엔진을 정의한다.

------------------------------------------------------------------------

## 2. 역할 정의

Executor = "입력을 받아 실행하고, 결과를 만들어 상태를 변경한다"

------------------------------------------------------------------------

## 3. 입력 / 출력

### 입력

-   task 정의
-   current state

### 출력

-   output
-   updated state

------------------------------------------------------------------------

## 4. 전체 실행 흐름

Scheduler → Guard (pre-check) → Executor → Guard (post-check) → Snapshot

------------------------------------------------------------------------

## 5. 내부 단계

### 5.1 Prepare

-   read scope 기준으로 state 추출
-   input normalize

### 5.2 Idempotency Check

-   fingerprint 생성
-   cache 존재 시 실행 생략

### 5.3 Execute

-   LLM 호출 또는 함수 실행

### 5.4 Output 검증

-   contract validate
-   schema check

### 5.5 State 적용

-   write scope에 따라 overwrite

### 5.6 반환

-   output, new_state 반환

------------------------------------------------------------------------

## 6. Execution Fingerprint

fingerprint = hash(task_id + normalized_input + relevant_state_subset)

------------------------------------------------------------------------

## 7. 캐시 구조

.execution_cache/ ├── `<fingerprint>`{=html}.json

------------------------------------------------------------------------

## 8. Task 실행 모델

### Hybrid 모델 (권장)

-   deterministic task → function 실행
-   generative task → LLM 실행

------------------------------------------------------------------------

## 9. 설계 원칙

-   Executor는 판단하지 않는다
-   Scheduler가 순서를 결정한다
-   Guard가 규칙을 강제한다
-   Executor는 실행만 담당한다

------------------------------------------------------------------------

## 10. 최소 구현 예시

``` python
def execute(task, state):
    input_data = build_input(task, state)

    fp = generate_fingerprint(task, input_data, state)

    if cache.exists(fp):
        return cache.load(fp)

    output = run_task(task, input_data)

    validate_output(output)

    new_state = apply_state(state, task["write"], output)

    cache.save(fp, output)

    return new_state
```

------------------------------------------------------------------------

## 11. 금지 사항

-   state 전체 접근
-   append 기반 state 변경
-   validation 없는 실행
-   cache 없는 retry

------------------------------------------------------------------------

## 12. 결론

Executor는 시스템의 실행 엔진이다.

이 레이어가 완성되어야: - Scheduler - Guard - Snapshot - Idempotency

모든 구조가 실제로 동작하기 시작한다.
