# AXON 설치 및 실행 가이드 (Installation & Execution) 🏭

AXON은 백엔드 데몬(Rust)과 프론트엔드 스튜디오(React/Vite)로 구성된 자동화 소프트웨어 공장입니다.

## 1. 사전 준비 (Prerequisites)

AXON은 로컬 인프라(Ollama)와 클라우드 API(Gemini)를 모두 지원합니다. 프로젝트 성격에 맞춰 선택하십시오.

### 옵션 A: 로컬 LLM (Ollama) - 프라이버시 중심
1.  **Ollama 설치**: [ollama.com](https://ollama.com)에서 설치.
2.  **모델 다운로드**:
    ```bash
    ollama pull mistral  # 아키텍트/시니어용 추천
    ollama pull llama3   # 주니어용 추천
    ```

### 옵션 B: 클라우드 LLM (Google Gemini) - 성능 중심
1.  **API 키 발급**: [Google AI Studio](https://aistudio.google.com/)에서 API 키 생성.
2.  **환경 변수 설정**:
    ```bash
    export GEMINI_API_KEY="your-api-key-here"
    ```

*팁: `axon_config.json`에서 각 요원(Junior/Senior/Architect)별로 서로 다른 엔진을 할당할 수도 있습니다.*

## 2. 빌드 (Installation)

### 백엔드 (Rust)
```bash
# 전체 워크스페이스 빌드
cargo build --release
```
빌드가 완료되면 실행 파일은 `./target/release/axon-daemon`에 위치합니다.

### 프론트엔드 (Studio)
백엔드 서버가 `studio/dist` 폴더의 정적 파일을 서빙하므로, 사전 빌드가 필요합니다.
```bash
cd studio
npm install
npm run build
cd ..
```

## 3. 실행 (Running)

### 일반 실행
```bash
# 대화형 모드로 실행
./target/release/axon-daemon run

# 명세서(Spec)를 지정하여 즉시 공장 가동
./target/release/axon-daemon run GEMINI.md
```

### 리소스 최적화 실행 (권장)
하스웰(Haswell) 등 구형 CPU나 GPU가 없는 환경에서는 병렬 일꾼 수를 제한하여 Ollama의 타임아웃을 방지해야 합니다.
```bash
# 병렬 워커를 1개로 제한하여 안정적으로 가동
./target/release/axon-daemon run GEMINI.md --workers 1
```

## 4. 시스템 서비스 등록 (Linux systemd)

시스템 재부팅 시에도 자동으로 실행되도록 설정하려면 아래 예시를 참고하십시오.

`/etc/systemd/system/axon.service`:
```ini
[Unit]
Description=AXON Automated Software Factory Daemon
After=network.target

[Service]
Type=simple
User=dogsinatas
WorkingDirectory=/home/dogsinatas/rust_project/axon
# Ollama가 로컬에서 구동 중이어야 함
ExecStart=/home/dogsinatas/rust_project/axon/target/release/axon-daemon run --workers 1
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## 5. 주요 CLI 옵션
- `run [SPEC]`: 데몬 실행 및 명세서 로드.
- `--workers <N>`: 동시 가동할 에이전트 스레드 수 (기본값: CPU 코어 수).
- `--port <PORT>`: 웹 대시보드 포트 변경 (기본값: 8080).

---
*주의: Ollama 서비스가 먼저 실행되고 있어야 에이전트들이 정상적으로 모집됩니다.*
