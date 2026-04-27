# 📄 PHASE_09 Persistence (AXON v0.0.17)

## 🎯 목적
Execution 및 Observability에서 생성된 **로그와 메트릭을 영구 저장**하여  
시스템 재시작 이후에도 상태와 학습을 유지한다.

---

## 🧠 핵심 개념

- 현재 상태: 메모리 기반 (휘발성)
- 목표 상태: 디스크 기반 (영속성)

---

## ⚙️ 저장 대상

### 1. Agent Metrics
- latency 기록
- 실패 횟수
- 성공 횟수

### 2. Execution Logs
- task
- 결과
- 상태 (RUNNING / FAILED)

### 3. Failure Logs
- error 메시지
- agent id
- role

---

## 🔧 저장 구조

```json
{
  "metrics": {
    "agent_id": {
      "latencies": [],
      "fails": 0,
      "success": 0
    }
  },
  "logs": []
}
```

---

## 🔧 구현 코드 (Python)

```python
import json
import os

class Persistence:
    def __init__(self, path="axon_state.json"):
        self.path = path
        self._init_file()

    def _init_file(self):
        if not os.path.exists(self.path):
            with open(self.path, "w") as f:
                json.dump({"metrics": {}, "logs": []}, f)

    def load(self):
        with open(self.path, "r") as f:
            return json.load(f)

    def save(self, data):
        with open(self.path, "w") as f:
            json.dump(data, f, indent=2)

    def update_metrics(self, metrics):
        data = self.load()
        data["metrics"] = metrics
        self.save(data)

    def append_log(self, log):
        data = self.load()
        data["logs"].append(log)

        # 로그 크기 제한
        if len(data["logs"]) > 1000:
            data["logs"] = data["logs"][-1000:]

        self.save(data)
```

---

## 🔧 Pipeline 연동

```diff
+ persistence.update_metrics(self._metrics)

+ persistence.append_log({
+   "task": task,
+   "result": result,
+   "status": status
+ })
```

---

## 🔁 동작 흐름

```
Execution → Observability
 → Metrics 생성
 → Persistence 저장
 → 다음 실행에서 재사용
```

---

## 📊 효과

- 재시작 후에도 성능 데이터 유지
- Adaptive Routing 지속 학습 가능
- 장애 분석 가능

---

## ⚠️ 주의 사항

- 파일 I/O 비용 존재
- 너무 잦은 저장은 성능 저하 가능
- 필요 시 batch 저장 고려

---

## 🚫 제약 조건

- 메트릭 조작 금지
- 로그 삭제 금지 (자동 제한만 허용)

---

## 🧠 AXON 연결

| Phase | 역할 |
|------|------|
| PHASE_09 | State Persistence |

---

## 📌 요약

이 단계는:

- 시스템 상태를 “기억”하게 만들고
- 실행 데이터를 축적하며
- Adaptive Routing을 지속 가능하게 만든다

AXON의 **기억 레이어**이다.
