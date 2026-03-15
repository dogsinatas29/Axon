# encoding: utf-8
# AXON Installation & Execution Guide 🏭

AXON은 백엔드 데몬(Rust)과 프론트엔드 스튜디오(React)로 구성되어 있습니다. 아래 단계를 따라 설치 및 실행하실 수 있습니다.

## 1. 빌드 (Installation)

### 백엔드 (Rust)
```bash
# 전체 워크스페이스 빌드
cargo build --release -p axon-daemon
```
빌드가 완료되면 실행 파일은 `target/release/axon-daemon`에 위치합니다.

### 프론트엔드 (Studio)
프론트엔드 정적 파일이 백엔드 서버를 통해 서빙되므로, 사전 빌드가 필요합니다.
```bash
cd studio
npm install
npm run build
cd ..
```
*백엔드 실행 시 `studio/dist` 디렉토리를 자동으로 감지하여 웹 대시보드를 서빙합니다.*

## 2. 실행 (Running)

### 환경 변수 설정
에이전트들이 실제 연산을 수행하려면 LLM API 키가 필요합니다.
```bash
export GEMINI_API_KEY="your-google-api-key"
```
*API 키를 설정하지 않으면 시뮬레이션용 Mock 드라이버로 동작합니다.*

### 데몬 기동
```bash
./target/release/axon-daemon run
```
기동 후 브라우저에서 `http://localhost:8080`으로 접속하여 관제 타워(Studio)를 확인하십시오.

## 3. 데몬 서비스 등록 (Linux systemd)
시스템 재부팅 시에도 자동으로 실행되도록 데몬으로 등록하려면 아래 설정을 사용하십시오.

`/etc/systemd/system/axon.service` 파일을 생성합니다:
```ini
[Unit]
Description=AXON Automated Software Factory Daemon
After=network.target

[Service]
Type=simple
User=dogsinatas
WorkingDirectory=/home/dogsinatas/rust_project/axon
Environment=GEMINI_API_KEY=your-google-api-key
ExecStart=/home/dogsinatas/rust_project/axon/target/release/axon-daemon run
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**서비스 활성화:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable axon
sudo systemctl start axon
```

## 4. 주요 CLI 명령어
- `axon run`: 데몬 실행 (API + Web UI + Worker)
- `axon init`: 새로운 프로젝트 초기화
- `axon read <path>`: `Architecture.md`를 읽어 즉시 태스크 생성
- `axon status`: 현재 가동 상태 확인
