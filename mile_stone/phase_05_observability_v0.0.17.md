# 📄 PHASE_05 Observability (AXON v0.0.17)

## 🎯 목적

Execution 단계에서 발생한 **시스템 상태와 성능 지표를 수집 및 보고**한다.  
이 단계는 AXON의 실행 상태를 외부에서 이해 가능하게 만든다.

---

## 🧠 핵심 개념

- Observability는 “부가 기능”이 아니라 **필수 시스템 출력**
- 실행 결과뿐 아니라 **과정 상태를 기록**
- 실패도 동일하게 기록 (숨김 금지)

---

## ⚙️ 수집 대상

### 1. Agent 상태
- role
- id
- status (OK / WARN / FAIL)
- latency (ms)

### 2. Execution Path
- 실제 실행된 순서 기록

### 3. Runtime Metrics
- total_duration
- eval_count
- eval_duration

(Ollama response 기반)

### 4. Queue 상태
- 현재 queue length

(MVP에서는 0 또는 단일 task)

### 5. Failure 정보
- 실패 발생 여부
- 실패 원인

---

## 🧠 구현 코드 (Python)

```python
import time

class Observability:
    def collect(self, execution_result, raw_responses):
        start = time.time()

        agents = []
        failures = []

        # Path 기반 agent 기록
        for role, agent_id in execution_result.get("path", []):
            agents.append({
                "role": role,
                "id": agent_id,
                "status": "OK",
                "latency": None  # MVP: 미측정
            })

        # Runtime metrics (마지막 응답 기준)
        metrics = {}
        if raw_responses:
            last = raw_responses[-1]
            metrics = {
                "total_duration": last.get("total_duration"),
                "eval_count": last.get("eval_count"),
                "eval_duration": last.get("eval_duration")
            }

        # Failure 처리
        if execution_result.get("status") != "RUNNING":
            failures.append({
                "reason": execution_result.get("reason", "unknown")
            })

        report = {
            "agents": agents,
            "execution_path": execution_result.get("path", []),
            "metrics": metrics,
            "queue": {
                "length": 1
            },
            "failures": failures
        }

        return report
```

---

## 🔧 사용 예시

```python
obs = Observability()

report = obs.collect(result, raw_responses)

print(report)
```

---

## 📊 출력 구조

```json
{
  "agents": [
    {"role": "junior", "id": "junior_0", "status": "OK"},
    {"role": "senior", "id": "senior_0", "status": "OK"},
    {"role": "architect", "id": "architect", "status": "OK"}
  ],
  "execution_path": [
    ["junior", "junior_0"],
    ["senior", "senior_0"],
    ["architect", "architect"]
  ],
  "metrics": {
    "total_duration": 123456,
    "eval_count": 45,
    "eval_duration": 98765
  },
  "queue": {
    "length": 1
  },
  "failures": []
}
```

---

## 🚫 제약 조건

- 실행 결과를 수정하지 않는다
- 실패를 숨기지 않는다
- metric을 추론하지 않는다 (응답 기반만 사용)

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_05_OBSERVABILITY | 상태 보고 |

---

## 📌 요약

이 단계는:

- 실행 결과를 “보이게” 만든다
- 성능과 실패를 기록한다
- 시스템을 디버깅 가능하게 만든다

AXON의 **관측 레이어** 역할을 수행한다.
