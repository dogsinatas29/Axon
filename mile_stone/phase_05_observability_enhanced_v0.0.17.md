# 📄 PHASE_05 Observability Enhanced (AXON v0.0.17)

## 🎯 목적

Execution 전 과정의 **정확한 성능, 지연, 실패 데이터**를 수집하여  
시스템 상태를 정량적으로 분석 가능하게 만든다.

---

## 🧠 핵심 개념

- Observability는 단순 로그가 아니라 **측정 시스템**
- 각 Agent 호출 단위로 latency 측정
- Retry / Fallback / Timeout 모두 기록

---

## ⚙️ 수집 대상 (확장)

### 1. Agent Metrics (핵심)
- role
- id
- status (OK / FAIL)
- latency (ms)
- attempts (retry 횟수)
- fallback_used (boolean)

---

### 2. Execution Metrics
- total_duration (전체 실행 시간)
- step_count
- success / fail 여부

---

### 3. Runtime Metrics (Ollama 기반)
- eval_count
- eval_duration
- total_duration

---

### 4. Failure Trace
- 어느 role에서 실패했는지
- 어떤 agent였는지
- 에러 메시지

---

## 🧠 설계 변경 (핵심 포인트)

👉 기존: raw_responses 기반  
👉 변경: **Pipeline 내부에서 직접 기록**

---

## 🔧 Pipeline 변경 (핵심)

```diff
class ExecutionPipeline:
    def __init__(self, runtime):
        ...
+       self._trace = []

    def _generate(self, agent, prompt):
+       import time
+       start = time.time()

        last_error = None
+       attempts = 0

        for attempt in range(self.max_retries + 1):
            try:
                res = self.runtime.generate(
                    agent["model"],
                    prompt,
                    timeout=self.timeout
                )

                if "response" not in res:
                    raise ValueError("Invalid response format")

+               latency = (time.time() - start) * 1000

+               self._trace.append({
+                   "agent": agent["id"],
+                   "role": agent["role"],
+                   "status": "OK",
+                   "latency": latency,
+                   "attempts": attempt + 1
+               })

                return res["response"]

            except Exception as e:
                last_error = str(e)
+               attempts += 1

                if attempt == self.max_retries:
+                   latency = (time.time() - start) * 1000
+
+                   self._trace.append({
+                       "agent": agent["id"],
+                       "role": agent["role"],
+                       "status": "FAIL",
+                       "latency": latency,
+                       "attempts": attempts,
+                       "error": last_error
+                   })

                    raise RuntimeError(
                        f"TIMEOUT_OR_FAIL | agent={agent['id']} | error={last_error}"
                    )
```

---

## 🔧 Observability 클래스 (개선)

```python
class Observability:
    def collect(self, execution_result, trace):
        agents = []
        failures = []

        for t in trace:
            agents.append({
                "role": t["role"],
                "id": t["agent"],
                "status": t["status"],
                "latency": round(t["latency"], 2),
                "attempts": t["attempts"]
            })

            if t["status"] == "FAIL":
                failures.append({
                    "agent": t["agent"],
                    "error": t.get("error")
                })

        total_duration = sum(t["latency"] for t in trace)

        return {
            "agents": agents,
            "summary": {
                "total_duration_ms": round(total_duration, 2),
                "steps": len(trace),
                "status": execution_result.get("status")
            },
            "failures": failures
        }
```

---

## 📊 출력 예시

```json
{
  "agents": [
    {
      "role": "junior",
      "id": "junior_0",
      "status": "OK",
      "latency": 120.5,
      "attempts": 1
    }
  ],
  "summary": {
    "total_duration_ms": 340.2,
    "steps": 3,
    "status": "RUNNING"
  },
  "failures": []
}
```

---

## 🚫 제약 조건

- metric 추정 금지 (실측만 허용)
- 실패 숨김 금지
- trace 수정 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_05_OBSERVABILITY | 정밀 상태 측정 |

---

## 📌 요약

이 단계는:

- 실행을 “보이게” 만드는 것을 넘어서
- **측정 가능하게 만든다**

AXON의 **디버깅 / 운영 핵심 레이어**이다.
