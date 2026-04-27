# 📄 PHASE_07 Multi-Worker (AXON v0.0.17)

## 🎯 목적

단일 워커 구조에서 벗어나, 여러 Worker가 동시에 Task를 처리하도록 하여  
시스템의 처리량(throughput)을 향상시킨다.

---

## 🧠 핵심 개념

- Worker = 독립 실행 루프
- 여러 Worker가 동일 Queue를 공유
- 병렬 처리로 처리 속도 증가

---

## ⚙️ 설계 원칙

- Queue는 단일 공유 구조
- Worker는 독립적으로 task 소비
- race condition 방지 필요

---

## 🔧 구현 코드 (Python)

```python
import threading
import time
from collections import deque

class MultiWorkerScheduler:
    def __init__(self, pipeline, observability, worker_count=2):
        self.queue = deque()
        self.pipeline = pipeline
        self.observability = observability
        self.worker_count = worker_count
        self.running = False
        self.lock = threading.Lock()

    def submit(self, task):
        with self.lock:
            self.queue.append(task)

    def start(self, context):
        self.running = True
        self.workers = []

        for i in range(self.worker_count):
            t = threading.Thread(target=self._worker_loop, args=(context,), daemon=True)
            t.start()
            self.workers.append(t)

    def stop(self):
        self.running = False

    def _get_task(self):
        with self.lock:
            if not self.queue:
                return None
            return self.queue.popleft()

    def _worker_loop(self, context):
        while self.running:
            task = self._get_task()

            if task is None:
                time.sleep(0.05)
                continue

            # trace 초기화
            self.pipeline._trace = []

            result = self.pipeline.run(context, task)

            report = self.observability.collect(
                result,
                self.pipeline._trace
            )

            self._handle_result(task, result, report)

    def _handle_result(self, task, result, report):
        print("\n[WORKER RESULT]")
        print("Task:", task)
        print("Result:", result)
        print("Report:", report)
```

---

## 🔁 동작 흐름

```
submit → queue
 ↓
worker1 → task1 처리
worker2 → task2 처리
worker3 → task3 처리
```

---

## ⚠️ 핵심 이슈 (중요)

### 1. Lock 필요
- queue 접근 보호

### 2. Trace 공유 문제
- pipeline._trace는 worker별로 분리 필요 (향후 개선)

### 3. 순서 보장 없음
- 결과 순서 뒤섞임 (정상)

---

## 🚫 제약 조건

- shared state 최소화
- worker 간 데이터 공유 금지
- ordering 보장 없음

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_07_MULTI_WORKER | 병렬 처리 |

---

## 📌 요약

이 단계는:

- 처리량을 증가시키고
- 병렬 실행을 가능하게 하며
- 시스템을 실제 운영 수준으로 끌어올린다

AXON의 **성능 확장 레이어**이다.
