# AXON 설치 및 실행 가이드 / Installation & Execution Guide 🏭

AXON은 백엔드 데몬(Rust)과 프론트엔드 스튜디오(React/Vite)로 구성된 자동화 소프트웨어 공장입니다.  
AXON is an automated software factory consisting of a backend daemon (Rust) and a frontend studio (React/Vite).

---

## 1. 사전 준비 / Prerequisites

AXON은 로컬 인프라(Ollama)와 클라우드 API(Gemini)를 모두 지원합니다.  
AXON supports both local infrastructure (Ollama) and cloud APIs (Gemini).

### 옵션 A: 로컬 LLM (Ollama) / Option A: Local LLM (Ollama)
*프라이버시 및 오프라인 환경 중심 / Focus on privacy and offline environments*

1.  **Ollama 설치 / Install Ollama**: [ollama.com](https://ollama.com)
2.  **모델 다운로드 / Download Models**:
    ```bash
    ollama pull mistral  # 아키텍트/시니어 추천 (Recommended for Architect/Senior)
    ollama pull llama3   # 주니어 추천 (Recommended for Junior)
    ```

### 옵션 B: 클라우드 LLM (Google Gemini) / Option B: Cloud LLM (Google Gemini)
*고성능 및 대규모 컨텍스트 중심 / Focus on high performance and large context*

1.  **API 키 발급 / Get API Key**: [Google AI Studio](https://aistudio.google.com/)
2.  **환경 변수 설정 / Set Environment Variable**:
    ```bash
    export GEMINI_API_KEY="your-api-key-here"
    ```

---

## 2. 빌드 / Installation & Build

### 백엔드 빌드 / Backend Build (Rust)
```bash
cargo build --release
```
*실행 파일 위치 / Binary path: `./target/release/axon-daemon`*

### 프론트엔드 빌드 / Frontend Build (Studio)
```bash
cd studio
npm install
npm run build
cd ..
```
*백엔드가 `studio/dist`를 서빙합니다 / The backend serves `studio/dist`.*

---

## 3. 실행 / Running AXON

### 일반 실행 / Basic Run
```bash
# 대화형 모드 / Interactive mode
./target/release/axon-daemon run

# 명세서 지정 실행 / Run with specific specification
./target/release/axon-daemon run GEMINI.md
```

### 리소스 최적화 실행 / Resource Optimized Run (Recommended)
구형 CPU(하스웰 등)나 GPU가 없는 환경에서는 병렬 작업 수를 제한하십시오.  
Limit the number of parallel workers for older CPUs or environments without GPUs.
```bash
# 병렬 워커를 1개로 제한 / Limit to 1 parallel worker
./target/release/axon-daemon run GEMINI.md --workers 1
```

---

## 4. 시스템 서비스 등록 / Systemd Service Setup (Linux)

`/etc/systemd/system/axon.service`:
```ini
[Unit]
Description=AXON Automated Software Factory Daemon
After=network.target

[Service]
Type=simple
User=your-username
WorkingDirectory=/path/to/axon
ExecStart=/path/to/axon/target/release/axon-daemon run --workers 1
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

---

## 5. 주요 CLI 옵션 / Key CLI Options
- `run [SPEC]`: 데몬 실행 및 명세서 로드 / Start daemon and load specification.
- `--workers <N>`: 동시 가동할 에이전트 수 / Number of parallel agents (Default: CPU cores).
- `--port <PORT>`: 웹 대시보드 포트 / Web dashboard port (Default: 8080).

---
*Created by Antigravity AI Coding Assistant.*
