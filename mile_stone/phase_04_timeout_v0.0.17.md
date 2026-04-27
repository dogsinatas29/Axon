# 📄 PHASE_04 Timeout Control (AXON v0.0.17)

## 🎯 목적

Execution 단계에서 각 LLM 호출에 대해 **최대 허용 시간(timeout)** 을 설정하여  
응답 지연 또는 hang 상태를 강제로 차단한다.

---

## 🧠 핵심 개념

- Timeout은 **시간 기반 실패 제어**
- 지정 시간 초과 시 즉시 실패 처리
- Retry / Fallback과 결합되어 동작

---

## ⚙️ 처리 규칙

- 각 generate 호출에 timeout 적용
- timeout 발생 시 예외 발생 → retry 로직으로 전달
- retry 초과 시 fallback 실행
- fallback 모두 실패 시 FAILED 반환

---

## 🔧 변경 사항 (Diff)

```diff
import requests

 class ExecutionPipeline:
     def __init__(self, runtime):
         self.runtime = runtime
         self.max_retries = 2
+        self.timeout = 10  # seconds

     def _generate(self, agent, prompt):
         last_error = None

         for attempt in range(self.max_retries + 1):
             try:
-                res = self.runtime.generate(agent["model"], prompt)
+                res = self.runtime.generate(
+                    agent["model"],
+                    prompt,
+                    timeout=self.timeout
+                )

                 if "response" not in res:
                     raise ValueError("Invalid response format")

                 return res["response"]

             except Exception as e:
                 last_error = str(e)

                 if attempt == self.max_retries:
                     raise RuntimeError(
                         f"TIMEOUT_OR_FAIL | agent={agent['id']} | error={last_error}"
                     )
```

---

## 🔧 Runtime 수정 필요 (중요)

OllamaRuntime.generate에 timeout 전달:

```diff
 def generate(self, model, prompt, timeout=None):
-    r = requests.post(url, json=payload)
+    r = requests.post(url, json=payload, timeout=timeout)
```

---

## 🔁 동작 흐름

```
generate()
 → timeout 발생
 → retry (최대 2회)
 → fallback 시도
 → 전체 실패 시 FAILED
```

---

## 📊 상태 정의

- RUNNING: 정상 응답
- FAILED: timeout + fallback 실패
- BLOCKED: 레이어 조건 미충족

---

## 🚫 제약 조건

- timeout은 runtime 내부가 아니라 pipeline에서 제어
- 무한 대기 금지
- fallback 없이 단일 실패 허용 금지

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_04_EXECUTION | Timeout Control |

---

## 📌 요약

이 단계는:

- LLM 응답 지연을 제어
- 시스템 hang 방지
- retry / fallback과 결합되어 안정성 강화

AXON의 **시간 제어 레이어**를 구성한다.
