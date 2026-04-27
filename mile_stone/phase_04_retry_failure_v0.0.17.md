# 📄 PHASE_04 Execution Retry & Failure Handling (AXON v0.0.17)

## 🎯 목적

Execution 단계에서 발생하는 Runtime 실패를 제어하고,  
재시도(retry) 및 실패 처리(failure handling)를 수행한다.

---

## 🧠 핵심 개념

- Runtime 실패는 정상적인 상황으로 간주한다
- 실패는 반드시 기록되고 반환되어야 한다
- Retry는 Pipeline 레벨에서만 수행된다

---

## ⚙️ 처리 규칙

- 최대 재시도 횟수: 2 (기본값)
- 응답 검증 실패 시 재시도
- 재시도 초과 시 FAILURE 반환
- fallback 없음 (현재 단계)

---

## 🔧 변경 사항 (Diff)

```diff
class ExecutionPipeline:
    def __init__(self, runtime):
        self.runtime = runtime
+       self.max_retries = 2

    def _generate(self, agent, prompt):
-       return self.runtime.generate(agent["model"], prompt)["response"]
+       last_error = None
+
+       for attempt in range(self.max_retries + 1):
+           try:
+               res = self.runtime.generate(agent["model"], prompt)
+
+               if "response" not in res:
+                   raise ValueError("Invalid response format")
+
+               return res["response"]
+
+           except Exception as e:
+               last_error = str(e)
+
+               if attempt == self.max_retries:
+                   raise RuntimeError(
+                       f"RUNTIME_FAIL | agent={agent['id']} | error={last_error}"
+                   )
```

---

## 🔧 run() 수정

```diff
    def run(self, context, task):

        ...

        try:
            junior = self._select_agent(context, "junior")
            result = self._generate(junior, result)

            senior = self._select_agent(context, "senior")
            result = self._generate(senior, result)

            architect = self._select_agent(context, "architect")
            result = self._generate(architect, result)

        except Exception as e:
            return {
                "status": "FAILED",
                "reason": str(e),
                "path": path
            }
```

---

## 📊 상태 정의

- RUNNING: 정상 실행 완료
- FAILED: Runtime 실패 발생
- BLOCKED: 최소 레이어 조건 미충족

---

## 🚫 제약 조건

- Runtime에 retry 로직 추가 금지
- Agent 교체 금지
- Model 변경 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_04_EXECUTION | Retry & Failure Handling |

---

## 📌 요약

이 단계는:

- Runtime 실패를 제어
- 재시도 수행
- 실패를 구조적으로 반환

AXON의 **안정성 레이어**를 구성한다.
