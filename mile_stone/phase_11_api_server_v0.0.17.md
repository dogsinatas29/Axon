# 📄 PHASE_11 API Serverization (AXON v0.0.17)

## 🎯 목적
AXON 시스템을 외부에서 호출 가능한 **API 서버 형태**로 변환하여  
다른 서비스, UI, 클라이언트에서 사용할 수 있도록 한다.

---

## 🧠 핵심 개념

- AXON = 내부 엔진
- API Server = 외부 인터페이스
- HTTP 기반 요청/응답 구조

---

## ⚙️ 기본 구조

```
Client
 ↓
HTTP API (FastAPI)
 ↓
Scheduler → Execution → AXON Core
 ↓
Response 반환
```

---

## 🔧 기술 선택

- Framework: FastAPI
- Server: Uvicorn

---

## 🔧 기본 구현 코드

```python
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

# AXON 구성 요소 (이미 존재한다고 가정)
scheduler = None
context = None

class TaskRequest(BaseModel):
    task: str

@app.post("/submit")
def submit_task(req: TaskRequest):
    result = scheduler.submit(req.task)
    return result

@app.get("/health")
def health_check():
    return {"status": "OK"}

@app.get("/queue")
def queue_status():
    return {
        "length": len(scheduler.queue),
        "limit": scheduler.queue_limit
    }
```

---

## 🔧 서버 실행

```bash
uvicorn main:app --host 0.0.0.0 --port 8000
```

---

## 📡 API 목록

### 1. Task 제출

```
POST /submit
```

Request:

```json
{
  "task": "Explain AXON"
}
```

Response:

```json
{
  "status": "ACCEPTED",
  "queue_size": 3
}
```

---

### 2. 상태 확인

```
GET /health
```

---

### 3. Queue 상태

```
GET /queue
```

---

## 🔁 동작 흐름

```
Client 요청
 → API 수신
 → Scheduler.submit()
 → Queue 적재
 → Worker 처리
 → 결과 출력 (로그/추후 확장)
```

---

## ⚠️ 현재 한계 (중요)

- 결과를 즉시 반환하지 않음 (async 처리)
- polling 또는 callback 필요

---

## 🔧 확장 방향

### 1. Result 조회 API

```
GET /result/{task_id}
```

### 2. WebSocket 지원
- 실시간 결과 전달

### 3. 인증 (Auth)
- API Key / Token

---

## 🚫 제약 조건

- Execution 직접 호출 금지 (Scheduler 통해서만)
- Queue bypass 금지

---

## 🧠 AXON 연결

| Phase | 역할 |
|------|------|
| PHASE_11 | External Interface |

---

## 📌 요약

이 단계는:

- AXON을 외부 시스템과 연결하고
- HTTP 기반으로 노출하며
- 실제 서비스 형태로 전환한다

AXON의 **서비스 인터페이스 레이어**이다.
