# 📄 PHASE_03 Context Builder (AXON v0.0.17)

## 🎯 목적

ValidationResult를 기반으로 **ExecutionContext를 생성**한다.  
이 단계는 실행에 필요한 모든 데이터를 명시적으로 구성한다.

---

## 🧠 핵심 개념

- Validation 결과를 재해석하지 않는다
- OK / FAIL을 그대로 분리한다
- 실행 가능한 Agent는 명시적으로 분리한다 (AvailableAgents)

---

## ⚙️ 처리 규칙

- 모든 Agent 포함 (전체 목록 유지)
- OK 상태 Agent만 AvailableAgents에 포함
- FAIL/WARN은 Warnings로 이동
- 암묵적 추론 금지

---

## 🧠 구현 코드 (Python)

```python
class ContextBuilder:
    def build(self, config, validation_result):
        agents = []
        available = {}
        warnings = []

        # 전체 agent 구성
        for agent in validation_result["agents"]:
            role = agent["role"]
            status = agent["status"]

            agents.append(agent)

            if status == "OK":
                if role not in available:
                    available[role] = []
                available[role].append(agent)
            else:
                warnings.append(agent)

        context = {
            "agents": agents,
            "available_agents": available,
            "constraints": {
                "queue_limit": config["execution"]["review_queue_limit"],
                "sampling_rate": config["execution"]["sampling_rate"]
            },
            "warnings": warnings
        }

        return context
```

---

## 🔧 사용 예시

```python
builder = ContextBuilder()

context = builder.build(config, validation_result)

print(context)
```

---

## 📊 출력 구조

```json
{
  "agents": [...],
  "available_agents": {
    "architect": [...],
    "senior": [...],
    "junior": [...]
  },
  "constraints": {
    "queue_limit": 2,
    "sampling_rate": 0.5
  },
  "warnings": [...]
}
```

---

## 🚫 제약 조건

- 새로운 Agent 생성 금지
- Agent 제거 금지
- Validation 결과 수정 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_03_BOOTSTRAPPING | ExecutionContext 생성 |

---

## 📌 요약

이 Context Builder는:

- 실행 가능한 구조를 명확히 분리
- 실패 상태를 숨기지 않음
- 다음 Phase(EXECUTION)의 입력을 완전히 정의
