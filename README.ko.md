# AXON: 자동화 소프트웨어 공장 (Automated Software Factory) 🏭

AXON은 Rust로 작성된 고성능 실시간 에이전트 오케스트레이션 시스템입니다. 소프트웨어 개발 공정을 자동화된 멀티 에이전트 생산 라인으로 변환하는 **에이전트 운영체제(Agent OS)** 역할을 수행합니다.

## 🚀 핵심 철학
- **보드 중심 (Board as SSOT)**: 개발 보드가 모든 시스템 상태의 유일한 진실 공급원(Single Source of Truth)입니다.
- **계층적 지능**: Architect, Senior, Junior 등 권한 레벨과 페르소나가 명확한 에이전트 조직 체계를 가집니다.
- **제어와 격리 (Control & Isolation)**: 작업의 일시 정지/재개(Pause/Resume) 및 프로젝트 간의 엄격한 데이터 격리를 보장합니다.
- **락인 아키텍처 (Lock-in)**: 승인된 코드와 사양은 아키텍처 문서에 "Locked-in"되어 시스템의 흔들리지 않는 기반이 됩니다.

## 🏛️ 아키텍처 구조
AXON은 **Hub -> Cluster -> Node** 계층 구조를 따릅니다:
- **Hub (axon-daemon)**: 전체 공정을 총괄하는 중앙 제어 엔진.
- **Cluster (axon-dispatcher)**: 태스크 큐 관리 및 에이전트 할당 최적화.
- **Node (axon-agent)**: 각기 다른 페르소나와 LLM 드라이버를 가진 개별 작업 수행 단위.

## 🛠️ 주요 기능 (v0.0.12)
- **실시간 제어**: `tokio::sync::watch` 채널 기반의 전역 작업 일시 정지/재개 기능.
- **프로젝트 격리**: 별도의 스토리지 및 API 라우팅을 통한 멀티 프로젝트 지원.
- **영속성**: SQLite 기반의 스레드, 태스크, 포스트, 이벤트 로그 저장.
- **이벤트 기반**: 반응형 협업과 완벽한 추적성을 위한 전역 이벤트 버스.
- **Studio UI**: 웹 기반의 대시보드 및 관제 패널 (개발 중).

## 🏁 시작하기

### 사전 요구사항
- [Rust](https://www.rust-lang.org/) (최신 안정 버전)
- SQLite

### 설치
```bash
# 저장소 클론
git clone https://github.com/dogsinatas/axon.git
cd axon

# 프로젝트 빌드
cargo build --release
```

### 데몬 실행
```bash
cargo run -p axon-daemon -- run
```

## 📅 로드맵
- [x] 코어 오케스트레이션 엔진 (v0.1.0 POC)
- [x] 멀티 프로젝트 격리 및 실시간 제어 (v0.0.12)
- [ ] 적대적 페르소나 모드 (에이전트 간 비판 논쟁)
- [ ] 실시간 UI 스트리밍 (XTerm.js 연동)

## 📜 라이선스
GPL-3.0 - 상세 내용은 [LICENSE](LICENSE) 파일을 참조하세요.
