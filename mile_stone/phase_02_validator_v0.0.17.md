# 📄 PHASE_02 Validator (AXON v0.0.17)

## 🎯 목적

CONFIG 기반으로 정의된 Agent들이 실제 실행 가능한 상태인지 검증하여  
ValidationResult를 생성한다.

---

## 🧠 핵심 개념

Validation은 다음 기준으로 수행된다:

- Runtime에서 모델 존재 여부 확인
- 존재하면 OK
- 없으면 FAIL

---

## ⚙️ 판단 기준

```
model ∈ runtime.list_models() → OK
else → FAIL
```

---

## 🧠 구현 코드 (Python)

```python
class Validator:
    def __init__(self, runtime):
        self.runtime = runtime

    def validate(self, config):
        models = self.runtime.list_models()

        results = []

        # Architect
        arch = config["agents"]["architect"]
        results.append(self._check_agent("architect", arch, models))

        # Seniors
        for i, s in enumerate(config["agents"]["seniors"]):
            results.append(self._check_agent(f"senior_{i}", s, models))

        # Juniors
        for i, j in enumerate(config["agents"]["juniors"]):
            results.append(self._check_agent(f"junior_{i}", j, models))

        return {"agents": results}

    def _check_agent(self, role_id, agent, models):
        model = agent["model"]

        if model in models:
            return {
                "id": role_id,
                "role": role_id.split("_")[0],
                "status": "OK",
                "reason": "model available"
            }
        else:
            return {
                "id": role_id,
                "role": role_id.split("_")[0],
                "status": "FAIL",
                "reason": "model not found"
            }
```

---

## 🔧 사용 예시

```python
runtime = OllamaRuntime()
validator = Validator(runtime)

config = {
    "agents": {
        "architect": {"model": "mistral:latest"},
        "seniors": [{"model": "mistral:latest"}],
        "juniors": [{"model": "wrong-model"}]
    }
}

result = validator.validate(config)
print(result)
```

---

## 📊 출력 예시

```json
{
  "agents": [
    {"id":"architect","role":"architect","status":"OK","reason":"model available"},
    {"id":"senior_0","role":"senior","status":"OK","reason":"model available"},
    {"id":"junior_0","role":"junior","status":"FAIL","reason":"model not found"}
  ]
}
```

---

## 🚫 제약 조건

- Runtime 재시도 금지
- Model 자동 변경 금지
- Agent 수정 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_02_VALIDATION | ValidationResult 생성 |

---

## 📌 요약

이 Validator는:

- 단순하지만 결정적인 판단 수행
- 실행 가능 여부만 판별
- 이후 Phase의 기반 데이터 생성
