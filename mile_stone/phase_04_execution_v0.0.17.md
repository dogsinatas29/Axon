# 📄 PHASE_04 Execution Pipeline (AXON v0.0.17)

## 🎯 목적

ExecutionContext를 기반으로 **Task를 실제로 실행 파이프라인을 따라 처리**한다.

이 단계는 AXON에서 처음으로 실제 LLM 호출이 발생하는 단계이다.

---

## 🧠 핵심 개념

- Task는 반드시 역할 체인을 따라 흐른다:

  Task → Junior → Senior → Architect

- 각 단계는 이전 결과를 입력으로 사용한다
- 역할은 절대 변경되지 않는다

---

## 🔒 최소 레이어 강제 조건 (Hard Constraint)

다음 조건이 만족되지 않으면 실행 차단:

- Junior layer: OK agent ≥ 1
- Senior layer: OK agent ≥ 1

조건 미충족 시:

```
status: BLOCKED
reason: Minimum layer requirement not satisfied
```

---

## ⚙️ 처리 규칙

- 각 role에서 정확히 1개의 agent만 선택 (MVP)
- round-robin 미사용 (추후 확장)
- retry 없음 (추후 PHASE 확장)
- fallback 없음
- runtime은 순수 실행만 수행

---

## 🧠 실행 흐름

```
Task
 ↓
Junior (generate)
 ↓
Senior (review)
 ↓
Architect (finalize)
 ↓
Result
```

---

## 🧠 구현 코드 (Python)

```python
class ExecutionPipeline:
    def __init__(self, runtime):
        self.runtime = runtime

    def run(self, context, task):
        # 1. 최소 레이어 검증
        if not self._check_minimum_layer(context):
            return {
                "status": "BLOCKED",
                "reason": "Minimum layer requirement not satisfied"
            }

        path = []
        result = task

        # 2. Junior
        junior = self._select_agent(context, "junior")
        result = self._generate(junior, result)
        path.append(("junior", junior["id"]))

        # 3. Senior
        senior = self._select_agent(context, "senior")
        result = self._generate(senior, result)
        path.append(("senior", senior["id"]))

        # 4. Architect
        architect = self._select_agent(context, "architect")
        result = self._generate(architect, result)
        path.append(("architect", architect["id"]))

        return {
            "status": "RUNNING",
            "result": result,
            "path": path
        }

    def _check_minimum_layer(self, context):
        return (
            len(context["available_agents"].get("junior", [])) > 0 and
            len(context["available_agents"].get("senior", [])) > 0
        )

    def _select_agent(self, context, role):
        return context["available_agents"][role][0]

    def _generate(self, agent, prompt):
        return self.runtime.generate(agent["model"], prompt)["response"]
```

---

## 🔧 사용 예시

```python
pipeline = ExecutionPipeline(runtime)
result = pipeline.run(context, "Explain AXON in one sentence")

print(result)
```

---

## 📊 출력 구조

```json
{
  "status": "RUNNING",
  "result": "...",
  "path": [
    ["junior", "junior_0"],
    ["senior", "senior_0"],
    ["architect", "architect"]
  ]
}
```

---

## 🚫 제약 조건

- 역할 변경 금지
- 모델 자동 선택 금지
- Validation 무시 금지
- Execution 순서 변경 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_04_EXECUTION | Task 실행 |

---

## 📌 요약

이 단계는:

- 실제 LLM 호출 수행
- 구조를 유지한 채 결과 생성
- 최소 단위 실행 흐름 검증

AXON의 “실행 엔진” 역할을 수행한다.
