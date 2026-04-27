# 📄 PHASE_10 Feedback Loop (AXON v0.0.17)

## 🎯 목적
Persistence에 저장된 메트릭을 활용하여  
**Routing 전략을 자동으로 조정**하는 피드백 루프를 구축한다.

---

## 🧠 핵심 개념

- Observability → 데이터 수집
- Adaptive Routing → 선택
- Feedback Loop → **전략 개선**

---

## ⚙️ 동작 구조

```
Execution
 → Observability
   → Persistence 저장
     → Feedback 분석
       → Routing 파라미터 업데이트
```

---

## 🔧 조정 대상

### 1. 실패 패널티 (fail_penalty)

- 실패가 많을수록 선택 확률 감소

### 2. latency 가중치 (latency_weight)

- 느린 agent 선택 확률 감소

---

## 🔧 기본 파라미터

```python
self._routing_params = {
    "latency_weight": 1.0,
    "fail_penalty": 1000
}
```

---

## 🔧 Feedback 로직

```python
def update_routing_params(self, metrics):
    total_fail = sum(m["fails"] for m in metrics.values())
    total_success = sum(m["success"] for m in metrics.values())

    if total_success == 0:
        return

    fail_ratio = total_fail / total_success

    # 실패율이 높으면 패널티 증가
    if fail_ratio > 0.3:
        self._routing_params["fail_penalty"] *= 1.2

    # 안정적이면 완화
    elif fail_ratio < 0.1:
        self._routing_params["fail_penalty"] *= 0.9
```

---

## 🔧 Adaptive Routing과 연결

```diff
def score(a):
    m = self._metrics.get(a["id"], None)

    if not m or m["success"] == 0:
        return float("inf")

    avg_latency = sum(m["latencies"]) / len(m["latencies"])

-   return avg_latency + (m["fails"] * 1000)
+   return (
+       avg_latency * self._routing_params["latency_weight"]
+       + (m["fails"] * self._routing_params["fail_penalty"])
+   )
```

---

## 🔁 실행 타이밍

- N번 실행마다 (예: 10 task)
- 또는 일정 시간 간격

---

## 📊 효과

- 실패 많은 agent 자동 회피
- 시스템 전체 latency 감소
- 환경 변화에 자동 적응

---

## ⚠️ 주의 사항

- 너무 빠른 조정은 불안정성 유발
- 최소 변화율 유지 필요

---

## 🚫 제약 조건

- 외부 개입 없이 자동 동작
- 메트릭 조작 금지
- 극단적 값 제한 필요

---

## 🧠 AXON 연결

| Phase | 역할 |
|------|------|
| PHASE_10 | Self Optimization |

---

## 📌 요약

이 단계는:

- 시스템이 스스로 학습하고
- 선택 전략을 개선하며
- 환경 변화에 적응하도록 만든다

AXON의 **자기 최적화 레이어**이다.
