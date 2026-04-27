# 📄 PHASE_06 Queue Control (AXON v0.0.17)

## 🎯 목적

Scheduler의 Task Queue에 대해 **입력 제한 및 제어(backpressure)** 를 적용하여  
시스템 과부하 및 메모리 폭증을 방지한다.

---

## 🧠 핵심 개념

- Queue는 무한히 증가하면 안 된다
- 입력 속도 > 처리 속도 상황을 제어해야 한다
- 초과 요청은 명시적으로 거부한다 (REJECTED)

---

## ⚙️ 제어 요소

### 1. Queue Limit (하드 제한)
- 최대 허용 Task 수 설정
- 초과 시 즉시 거부

### 2. Backpressure
- Queue가 가득 차면 submit 차단
- 호출자에게 상태 반환

### 3. Queue 상태 노출
- 현재 길이
- 최대 용량

---

## 🔧 변경 사항 (Diff)

```diff
class TaskScheduler:
     def __init__(self, pipeline, observability):
         self.queue = deque()
         self.pipeline = pipeline
         self.observability = observability
         self.running = False
+        self.queue_limit = 10

     def submit(self, task):
-        self.queue.append(task)
+        if len(self.queue) >= self.queue_limit:
+            return {
+                "status": "REJECTED",
+                "reason": "QUEUE_FULL"
+            }
+
+        self.queue.append(task)
+        return {
+            "status": "ACCEPTED",
+            "queue_size": len(self.queue)
+        }
```

---

## 📊 Observability 확장

```diff
"queue": {
-   "length": 1
+   "length": len(self.queue),
+   "limit": self.queue_limit
}
```

---

## 🔁 동작 흐름

```
submit()
 → queue 확인
   → 여유 있음 → ACCEPTED
   → 초과 → REJECTED
```

---

## 🧪 테스트 방법

```python
for i in range(20):
    print(scheduler.submit(f"task {i}"))
```

결과:

- 초기: ACCEPTED
- 초과: REJECTED

---

## 🚫 제약 조건

- 자동 대기 금지
- 자동 drop 금지
- retry는 외부 책임

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_06_SCHEDULER | Queue 제어 |

---

## 📌 요약

이 단계는:

- 무제한 입력을 차단
- 시스템 안정성 확보
- 과부하 상황을 명시적으로 처리

AXON의 **입력 제어 레이어**이다.
