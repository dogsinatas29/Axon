# 📄 PHASE_04 Fallback Handling (AXON v0.0.17)

## 🎯 목적

Execution 단계에서 특정 Agent 실행 실패 시,  
동일 role 내 다른 Agent로 **자동 전환(fallback)** 한다.

---

## 🧠 핵심 개념

- Fallback은 실패 대응 전략이다 (what if fail)
- Round-robin 위에서 동작한다
- 동일 role 내에서만 전환 가능
- 모든 후보 실패 시 최종 FAIL

---

## ⚙️ 처리 규칙

- primary agent 실패 시 다음 agent 시도
- 최대 후보 수만큼 시도
- 모든 후보 실패 시 FAILURE 반환
- retry 로직과 독립적으로 동작

---

## 🔧 변경 사항 (Diff)

```diff
 def _generate_with_fallback(self, context, role, prompt):
+    agents = context["available_agents"].get(role, [])
+
+    if not agents:
+        raise RuntimeError(f"NO_AVAILABLE_AGENT | role={role}")
+
+    last_error = None
+
+    # 모든 agent 순회 (fallback)
+    for i in range(len(agents)):
+        agent = self._select_agent(context, role)
+
+        try:
+            return self._generate(agent, prompt), agent
+
+        except Exception as e:
+            last_error = str(e)
+            continue
+
+    raise RuntimeError(
+        f"FALLBACK_FAIL | role={role} | error={last_error}"
+    )
```

---

## 🔧 run() 변경

```diff
- junior = self._select_agent(context, "junior")
- result = self._generate(junior, result)
+ result, junior = self._generate_with_fallback(context, "junior", result)

- senior = self._select_agent(context, "senior")
- result = self._generate(senior, result)
+ result, senior = self._generate_with_fallback(context, "senior", result)

- architect = self._select_agent(context, "architect")
- result = self._generate(architect, result)
+ result, architect = self._generate_with_fallback(context, "architect", result)
```

---

## 🔁 동작 예시

```
junior_0 (fail)
 → junior_1 (success)
```

---

## 📊 상태 정의

- RUNNING: 정상 실행
- FAILED: 모든 fallback 실패
- BLOCKED: 최소 레이어 미충족

---

## 🚫 제약 조건

- 다른 role로 fallback 금지
- model 변경 금지
- validation 무시 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_04_EXECUTION | Failure Recovery |

---

## 📌 요약

이 단계는:

- 실패 시 다른 agent로 자동 전환
- 실행 안정성 향상
- round-robin 기반 확장

AXON의 **복원력(resilience)**을 담당한다.
