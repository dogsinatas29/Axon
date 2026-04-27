# 📄 PHASE_04 Round-Robin Selection (AXON v0.0.17)

## 🎯 목적

Execution 단계에서 동일 역할(role)에 여러 Agent가 존재할 경우,  
**순환 선택(Round-robin)** 방식으로 Agent를 분산 사용한다.

---

## 🧠 핵심 개념

- Round-robin은 **선택 전략**이다 (who to pick)
- 상태 기반 인덱스를 사용하여 순차 선택
- Execution 흐름은 변경하지 않는다

---

## ⚙️ 처리 규칙

- role별로 독립적인 index 유지
- 호출 시마다 다음 agent 선택
- agent 목록이 비어 있으면 즉시 오류

---

## 🔧 변경 사항 (Diff)

```diff
class ExecutionPipeline:
     def __init__(self, runtime):
         self.runtime = runtime
         self.max_retries = 2
+        self._rr_index = {}  # role별 인덱스

     def _select_agent(self, context, role):
-        return context["available_agents"][role][0]
+        agents = context["available_agents"].get(role, [])
+
+        if not agents:
+            raise RuntimeError(f"NO_AVAILABLE_AGENT | role={role}")
+
+        # 초기화
+        if role not in self._rr_index:
+            self._rr_index[role] = 0
+
+        idx = self._rr_index[role]
+        agent = agents[idx % len(agents)]
+
+        # 다음 인덱스 업데이트
+        self._rr_index[role] = (idx + 1) % len(agents)
+
+        return agent
```

---

## 🔁 동작 예시

Agents (junior):

```
junior_0, junior_1, junior_2
```

실행 순서:

```
junior_0 → junior_1 → junior_2 → junior_0 → ...
```

---

## 🧪 테스트 방법

```python
pipeline = ExecutionPipeline(runtime)

for _ in range(5):
    result = pipeline.run(context, "Test")
    print(result["path"])
```

---

## ⚠️ 주의 사항

- pipeline 인스턴스를 재사용해야 한다
- 매 실행마다 새로 생성하면 round-robin이 동작하지 않는다

---

## 🚫 제약 조건

- fallback 없음 (다음 단계에서 추가)
- retry 로직과 분리 유지
- context 구조 변경 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_04_EXECUTION | Agent 분산 선택 |

---

## 📌 요약

이 단계는:

- 단일 agent 선택에서 벗어나
- 다중 agent 분산 실행을 가능하게 하며
- 이후 fallback 확장을 위한 기반을 제공한다
