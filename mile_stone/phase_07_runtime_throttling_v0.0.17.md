# 📄 PHASE_07 Runtime Throttling (AXON v0.0.17)

## 🎯 목적

멀티 워커 환경에서 Runtime(Ollama)의 과부하를 방지하기 위해  
동시 실행 요청 수를 제한한다.

---

## 🧠 핵심 개념

- Worker 수 ≠ 실제 처리 가능 수
- Runtime은 제한된 리소스를 가진다 (GPU / CPU)
- Semaphore를 사용하여 동시 요청 수를 제한한다

---

## ⚙️ 설계 방식

- global semaphore 사용
- generate 호출 전 acquire
- 완료 후 release

---

## 🔧 구현 코드 (Python)

```python
import threading

class ExecutionPipeline:
    def __init__(self, runtime, max_concurrent=2):
        self.runtime = runtime
        self.max_retries = 2
        self.timeout = 10

        # 🔥 핵심
        self._semaphore = threading.Semaphore(max_concurrent)

    def _generate(self, agent, prompt):
        last_error = None

        for attempt in range(self.max_retries + 1):
            try:
                # 🔥 동시성 제한
                with self._semaphore:
                    res = self.runtime.generate(
                        agent["model"],
                        prompt,
                        timeout=self.timeout
                    )

                if "response" not in res:
                    raise ValueError("Invalid response format")

                return res["response"]

            except Exception as e:
                last_error = str(e)

                if attempt == self.max_retries:
                    raise RuntimeError(
                        f"THROTTLED_FAIL | agent={agent['id']} | error={last_error}"
                    )
```

---

## 🔁 동작 흐름

```
Worker → generate 요청
 → semaphore 대기
   → 슬롯 있으면 실행
   → 없으면 대기
 → 실행 완료 후 release
```

---

## 📊 효과

- 동시 실행 수 제한
- GPU/CPU 과부하 방지
- timeout 감소
- 전체 안정성 상승

---

## ⚠️ 설정 가이드

| 환경 | 추천 값 |
|------|--------|
| CPU only | 1 |
| 저사양 GPU (1050Ti) | 2 |
| 고성능 GPU | 3~5 |

---

## 🚫 제약 조건

- semaphore는 pipeline 내부에서 관리
- runtime 직접 수정 금지
- worker 수와 독립적으로 동작

---

## 🧠 AXON 연결

| AXON Phase | 역할 |
|-----------|------|
| PHASE_07_MULTI_WORKER | Runtime 보호 |

---

## 📌 요약

이 단계는:

- 병렬 실행의 위험을 제어하고
- Runtime을 보호하며
- 안정적인 처리량을 유지한다

AXON의 **리소스 보호 레이어**이다.
