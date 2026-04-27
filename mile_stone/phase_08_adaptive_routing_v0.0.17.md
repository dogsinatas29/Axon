# 📄 PHASE_08 Adaptive Routing (AXON v0.0.17)

## 🎯 목적
Observability 데이터를 활용하여 **성능이 더 나은 Agent를 우선 선택**하도록 라우팅 전략을 개선한다.

---

## 🧠 핵심 개념
- Round-robin → 공정성
- Adaptive Routing → **성능 최적화**
- 최근 latency / 실패율 기반으로 가중치 적용

---

## ⚙️ 전략
- role별 agent 통계 유지
- 지표:
  - avg_latency_ms
  - fail_rate
  - success_count
- 점수(score) = latency 가중 + 실패 패널티

---

## 🔧 데이터 구조

```python
self._metrics = {
  "agent_id": {
    "latencies": [],
    "fails": 0,
    "success": 0
  }
}
```

---

## 🔧 선택 로직 (Diff)

```diff
 def _select_agent(self, context, role):
     agents = context["available_agents"].get(role, [])
     if not agents:
         raise RuntimeError(f"NO_AVAILABLE_AGENT | role={role}")

-    # round-robin
-    idx = self._rr_index[role]
-    agent = agents[idx % len(agents)]
-    self._rr_index[role] = (idx + 1) % len(agents)
-    return agent
+    def score(a):
+        m = self._metrics.get(a["id"], None)
+        if not m or m["success"] == 0:
+            return float("inf")  # cold start: 뒤로
+        avg_latency = sum(m["latencies"]) / len(m["latencies"])
+        fail_penalty = m["fails"] * 1000
+        return avg_latency + fail_penalty
+
+    # 가장 점수 낮은 agent 선택
+    return min(agents, key=score)
```

---

## 🔧 메트릭 업데이트 (Diff)

```diff
 def _record_success(self, agent_id, latency):
+    m = self._metrics.setdefault(agent_id, {"latencies": [], "fails": 0, "success": 0})
+    m["latencies"].append(latency)
+    if len(m["latencies"]) > 50:
+        m["latencies"].pop(0)
+    m["success"] += 1

 def _record_fail(self, agent_id):
+    m = self._metrics.setdefault(agent_id, {"latencies": [], "fails": 0, "success": 0})
+    m["fails"] += 1
```

---

## 🔧 generate 내부 연결 (요약)

- 성공 시 → `_record_success(agent_id, latency)`
- 실패 시 → `_record_fail(agent_id)`

---

## 🔁 동작 흐름

```
select (adaptive)
 → generate
   → success → metric update
   → fail → fallback + metric update
```

---

## ⚠️ 주의

- cold start 문제 존재 (초기 데이터 없음)
- 필요 시 initial random/rr 혼합 가능

---

## 🚫 제약
- 다른 role로 이동 금지
- 모델 변경 금지
- metric 조작 금지

---

## 🧠 AXON 연결

| Phase | 역할 |
|------|------|
| PHASE_08 | Intelligent Routing |

---

## 📌 요약
- 단순 분산 → 성능 기반 선택
- 느린 agent 자동 배제
- 시스템이 스스로 최적화

AXON의 **지능 레이어 시작점**
