# 📄 PHASE_06 Scheduler / Queue (AXON v0.0.17)

## 🎯 목적

단일 Task 실행에서 벗어나,  
여러 Task를 **순차/지속적으로 처리하는 스케줄링 시스템**을 구축한다.

---

## 🧠 핵심 개념

- Execution은 “한 번 실행”
- Scheduler는 “계속 실행”
- Queue 기반으로 Task를 관리

---

## ⚙️ 구성 요소

### 1. Task Queue
- FIFO 구조
- 대기 중인 작업 저장

### 2. Scheduler
- Queue에서 Task를 꺼냄
- ExecutionPipeline 호출

### 3. Worker Loop
- 지속적으로 Task 처리

---

## 🧠 기본 구조

```
Task Queue
   ↓
Scheduler Loop
   ↓
ExecutionPipeline
   ↓
Observability
```

---

## 🔧 구현 코드 (Python)

```python
import time
from collections import deque

class TaskScheduler:
    def __init__(self, pipeline, observability):
        self.queue = deque()
        self.pipeline = pipeline
        self.observability = observability
        self.running = False

    def submit(self, task):
        self.queue.append(task)

    def run(self, context):
        self.running = True

        while self.running:
            if not self.queue:
                time.sleep(0.1)
                continue

            task = self.queue.popleft()

            # trace 초기화
            self.pipeline._trace = []

            result = self.pipeline.run(context, task)

            report = self.observability.collect(
                result,
                self.pipeline._trace
            )

            self._handle_result(task, result, report)

    def stop(self):
        self.running = False

    def _handle_result(self, task, result, report):
        print("\n=== TASK RESULT ===")
        print("Task:", task)
        print("Result:", result)
        print("Report:", report)
```

---

## 🔧 사용 예시

```python
scheduler = TaskScheduler(pipeline, obs)

scheduler.submit("Explain AXON")
scheduler.submit("What is retry logic?")
scheduler.submit("Describe fallback")

scheduler.run(context)
```

---

## 📊 동작 흐름

```
submit → queue 적재
run → loop 시작
 → task 꺼냄
 → 실행
 → 결과 출력
 → 반복
```

---

## 🚫 제약 조건

- 병렬 처리 없음 (MVP)
- 우선순위 없음
- 큐 길이 제한 없음 (추후 추가)

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_06_SCHEDULER | Task 관리 및 실행 |

---

## 📌 요약

이 단계는:

- AXON을 “단발 실행기”에서
- “지속 실행 시스템”으로 전환

시스템의 **운영 레이어**를 담당한다.
