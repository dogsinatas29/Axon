# 📄 PHASE_07 Worker Isolation (AXON v0.0.17)

## 🎯 목적

Multi-Worker 환경에서 발생하는 **상태 공유 문제를 제거**하여  
각 Worker가 완전히 독립적으로 실행되도록 만든다.

---

## 🧠 핵심 문제

기존 구조:

```
self.pipeline._trace
```

👉 모든 Worker가 동일한 pipeline 인스턴스를 공유  
👉 trace 데이터 충돌 / 오염 발생

---

## 🧠 해결 전략

> **Worker마다 Pipeline 인스턴스를 분리한다**

---

## ⚙️ 변경 원칙

- pipeline 공유 금지
- worker별 독립 인스턴스 생성
- trace는 instance 내부 상태로 유지

---

## 🔧 변경 사항 (Diff)

```diff
class MultiWorkerScheduler:
     def __init__(self, pipeline, observability, worker_count=2):
-        self.pipeline = pipeline
+        self.pipeline_factory = pipeline
         self.observability = observability
```

---

## 🔧 Worker 생성 시 Pipeline 분리

```diff
     def start(self, context):
         self.running = True
         self.workers = []

         for i in range(self.worker_count):
-            t = threading.Thread(target=self._worker_loop, args=(context,), daemon=True)
+            pipeline = self.pipeline_factory()  # 새 인스턴스 생성
+            t = threading.Thread(
+                target=self._worker_loop,
+                args=(context, pipeline),
+                daemon=True
+            )
             t.start()
             self.workers.append(t)
```

---

## 🔧 Worker Loop 수정

```diff
-    def _worker_loop(self, context):
+    def _worker_loop(self, context, pipeline):

         while self.running:
             task = self._get_task()

             if task is None:
                 time.sleep(0.05)
                 continue

-            self.pipeline._trace = []
+            pipeline._trace = []

-            result = self.pipeline.run(context, task)
+            result = pipeline.run(context, task)

             report = self.observability.collect(
                 result,
-                self.pipeline._trace
+                pipeline._trace
             )
```

---

## 🔁 동작 변화

### Before

```
Worker1 ─┐
Worker2 ─┼── shared pipeline → 충돌
Worker3 ─┘
```

---

### After

```
Worker1 → Pipeline A
Worker2 → Pipeline B
Worker3 → Pipeline C
```

👉 완전 분리

---

## ⚠️ 추가 고려사항

### Runtime 공유 여부

- runtime은 공유 가능 (stateless일 경우)
- 문제 발생 시 runtime도 분리 필요

---

## 🚫 제약 조건

- pipeline 공유 금지
- trace 공유 금지
- worker 간 상태 공유 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_07_MULTI_WORKER | Isolation |

---

## 📌 요약

이 단계는:

- 병렬 실행에서 발생하는 데이터 충돌 제거
- Worker 간 완전 독립성 확보
- 안정적인 멀티 워커 기반 구축

AXON의 **동시성 안정성 레이어**이다.
