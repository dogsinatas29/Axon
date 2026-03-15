# encoding: utf-8
# 🚀 Implementation Plan: AXON v0.1 Finalization

---

## 📅 Milestone: v0.1.0 - Full Lifecycle Automation
**"모든 이벤트는 추적되며, 모든 에이전트는 반응한다."**

### 1. 🏗️ Event System Formalization (Event Bus & Schema)
- [ ] **axon-core**: `EventType` 확장 및 `Event` 구조체 고도화 (`project_id`, `payload` 추가).
- [ ] **axon-storage**: `events` 테이블 생성 및 이벤트 영속화 기능 추가.
- [ ] **axon-daemon**: 전역 `EventBus`를 각 모듈로 전파.

### 2. 🧠 Reactive Agent Runtime (Listener Pattern)
- [ ] **axon-agent**: `AgentRuntime::run()` 루프 구현.
- [ ] **Subscription Logic**: 에이전트 역할별 이벤트 리스너 탑재.
  - `Junior`: `TASK_ASSIGNED` -> `process_task` -> `POST_ADDED`.
  - `Senior`: `POST_ADDED` (Junior) -> `review_task` -> `POST_ADDED` (Review).
  - `Architect`: `POST_ADDED` (Senior) -> `validate_arch` -> `ARCH_UPDATED`.
- [ ] **Dispatcher Interaction**: 스케줄러가 보낸 신호를 이벤트 버스로 전환.

### 3. 👥 Hierarchical Org Management (Hiring & Succession)
- [ ] **API Endpoints**: `/api/agents/hire`, `/api/agents/fire` 구현.
- [ ] **Succession Logic**: 에이전트 해고 시 하위 주니어를 다른 시니어에게 자동 재할당하는 로직 (`Succession Policy`).
- [ ] **Storage Update**: `agents` 테이블 관계성(FK) 검증 강화.

### 4. 📺 Real-time UI & Streaming (AXON Studio)
- [ ] **WebSocket Expansion**: 모든 시스템 로그 및 에이전트 채팅을 실시간으로 브로드캐스팅.
- [ ] **Live Threading**: 웹 뷰어에서 정적 리프레시 없이 새로운 포스트를 즉시 렌더링.

---

## 🛠️ 기술적 변경 순서 (Step-by-Step)

1. **Phase 1: Core & Storage Update**
   - `axon-core`의 `Event` 구조체 수정.
   - `axon-storage` 마이그레이션 (`events` 테이블).

2. **Phase 2: Agent Reactivity**
   - `AgentRuntime`이 `EventBus`를 구독하고 루프를 돌도록 수정.
   - `Daemon`에서 에이전트 런타임들을 관리하는 워커 풀 고도화.

3. **Phase 3: Org Management API**
   - 서버 핸들러 추가 및 스토리지 로직 연동.

4. **Phase 4: Final Validation**
   - 전체 워크플로우 테스트.
