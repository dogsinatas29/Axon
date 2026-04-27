# 📄 Ollama Runtime Adapter (AXON v0.0.17)

## 🎯 목적

AXON 시스템에서 Local Runtime (Ollama)을 호출하기 위한 최소 실행 어댑터.

이 어댑터는 다음 역할만 수행한다:

- 모델 목록 조회 (PHASE_02)
- 텍스트 생성 요청 (PHASE_04)
- 연결 상태 확인 (Health Check)

---

## ⚠️ 설계 원칙

- 상태를 저장하지 않는다
- 재시도 로직을 포함하지 않는다
- fallback 로직을 포함하지 않는다
- 순수 실행기 역할만 수행한다

---

## 🧠 구현 코드 (Python)

```python
import requests

class OllamaRuntime:
    def __init__(self, base_url="http://192.168.0.150:11434"):
        self.base_url = base_url

    def ping(self):
        r = requests.get(self.base_url)
        return r.status_code == 200

    def list_models(self):
        r = requests.get(f"{self.base_url}/api/tags")
        r.raise_for_status()
        return [m["name"] for m in r.json()["models"]]

    def generate(self, model, prompt):
        r = requests.post(
            f"{self.base_url}/api/generate",
            json={
                "model": model,
                "prompt": prompt,
                "stream": False
            }
        )
        r.raise_for_status()
        return r.json()
```

---

## 🔧 사용 예시

```python
runtime = OllamaRuntime()

# 1. 연결 확인
print(runtime.ping())

# 2. 모델 조회
print(runtime.list_models())

# 3. 생성 요청
res = runtime.generate("mistral:latest", "AXON test")
print(res["response"])
```

---

## 🧠 AXON 연결

| AXON Phase | 사용 메서드 |
|-----------|------------|
| PHASE_02_VALIDATION | list_models() |
| PHASE_04_EXECUTION | generate() |
| Health Check | ping() |

---

## 📌 요약

이 어댑터는:

- AXON과 Ollama 사이의 최소 인터페이스
- 정책 없이 실행만 담당
- 상위 시스템(Validator, Execution)이 제어를 담당
