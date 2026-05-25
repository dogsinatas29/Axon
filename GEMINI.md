# Principles

> 💡 **AXON의 절대 정체성**
> "AGI IDE 판타지"를 완전히 버리고, **"레거시를 죽이지 않고 점진적으로 진화시키는 도구 (Deterministic Runtime-Safe Software Evolution Kernel)"**

🚀 [LLM 코딩 원칙]
LLM Coding Principles: 
1. [Think Before Coding] 코딩 전 사고: 추측하지 마라. 요구사항이 모호하면 즉시 질문하고, 접근 방식과 트레이드오프(장단점)를 먼저 제시하라. 항상 가장 단순한 해결책부터 제안한다.
2. [Simplicity First] 단순성 우선: 코드는 최소한으로 짠다. 요청하지 않은 기능이나 추상화를 추가하지 마라. 코드의 가독성과 효율성만 유지한다.
3. [Minimal Changes] 최소한의 변경: 정밀 타격하라. 전체를 새로 쓰지 말고 필요한 부분만 정확히 수정한다. 기존 스타일을 유지하며, 내가 새로 만든 코드 중 사용되지 않는 것만 정리한다.
4. [Goal-Oriented Execution] 목표 중심 실행: 목표 → 계획 → 구현 → 검증 순서를 엄수한다. 검증 가능한 목표를 정의하고, 단계별 계획을 세우며, 성공 기준을 명확히 확인한다.
5. [No Hallucinated APIs] API 환각 금지: 존재하지 않는 API, 함수, 라이브러리를 날조하지 마라. 확실하지 않으면 반드시 질문한다.
6. [Stable Code Protection] 안정 코드 보호: 이미 검증된 코드는 건드리지 마라. 오직 직접적으로 요청받은 부분만 수정하며, 가능하면 변경 사항은 diff 형식으로 제시한다.
7. [Context Confirmation] 맥락 확인: 코드를 수정하기 전 반드시 맥락을 확인하라. 추측해서 때려 맞추지 말고, 누락된 코드나 파일이 있다면 당당하게 요청하라.
8. [Language Consistency] 언어 설정: 사용자와의 모든 소통(응답, 피드백, 문서화 등)은 기본적으로 한국어를 사용하며, 명시적인 요청이 없는 한 한국어 응답 원칙을 고수한다.

🚀 [작업 원칙]
1. 마일스톤별 md파일로 버전별 개발 구조로 작업방법을 확정. 
 - 모든 마일스톤 문서를 읽어들이면 바로 GEMINI.md의 코딩 원칙을 적용할 것
 - 마일스톤 문서를 읽어들이면 즉시 해당 버전에 대한 작업 계획(Implementation Plan)과 TODO 리스트를 작성할 것. 
 - 해당 작업 중 발생하는 모든 릴리즈 노트와 생성 결과물은 마일스톤의 버전을 따라 컴파일할 것

2. 릴리즈 노트는 release_note 폴더에서 별도 관리할 것 
 - 릴리즈 노트에 기록된 내용은 버전.md파일을 기반으로 작업한 내용 + 작업중 추가한 요소로 반영하고 릴리즈 노트가 완성되면 버전.md파일에 추가 작업 내용을 바로 기록할것
 - 릴리즈 노트의 파일 명은 v버전_release_notes.md로 통일할 것

3. 문서 하단의 # Repository Index — Architecture Navigation Layer 내용을 참조 하여 작업할 것. 
 - IR crates와 validator 분리되어 있으니 참조한 파일에서 해당 작업을 진행할 것. 
 - IR crates와 validator 등이 추가되거나 분리, 혹은 트리 구조의 변경이 있는 경우 반드시, 이 문서의 내용을 업데이트 하기 전 변경전 항목은 취소선을 긋고 새로 추가할 것. 

 # 📌 강제 파일 투사 규칙
- [트리거]: 프롬프트 입력 및 파일 최종 Write 직전 예외 없이 수행.
- [수행 명령]: 프로젝트 루트의 `context/` 폴더 내에 `yyyy.mm.dd.tt.synapse_gate.txt` 형태로 평문 파일을 생성하거나 Append(이어쓰기)하라.
- [필수 기록 데이터]: 
  1. 이번 프롬프트로 인해 수행한 추론 과정 요약 (Core CoT)
  2. 수정 및 생성된 변경 파일 목록 (Affected Files Diff)
- [주의 사항]: 구글 안티그래비티 내부 암호화 샌드박스에 갇히지 말고, 반드시 외부에서 인간 매니저와 AXON 데몬이 읽을 수 있는 평문(Plain Text)으로 출력할 것.

📜 마일스톤 문서 생성 및 경로 규격 (제미나이.md 추가 사양)
1. 표준 저장 경로 (Standard Path)
- 모든 마일스톤 문서는 프로젝트 루트를 기준으로 다음의 엄격한 경로 규칙을 따른다.
Path: ~/언어_프로젝트/프로젝트명/mile_stone/v[버전명].md
예시: ~/python_antigravity/synapse/milestone/v0.2.20.md
2. 자동 생성 프로토콜 (Auto-Generation Protocol)
- 사용자가 **"내용 설명하고 이거 정리해서 버전 x.x.x.md로 만들어줘"**라고 요청할 경우, 제미나이는 즉시 다음 프로세스를 수행한다.
- 생성되는 모든 MD 파일 상단에 # encoding: utf-8을 명시하고, 저장 시 강제로 UTF-8(No BOM)로 지정
- Context 덤프: 대화 중 나온 모든 설계, 로직, 주의사항을 수집.
- 규격 적용: 아래의 [마일스톤 문서 표준 템플릿]에 맞춰 내용 정리.
- 파일 생성: 지정된 경로에 문서 생성 (혹은 내용 출력).
- 릴리즈 노트가 완료되면 해당 내용을 마일스톤버전 문서에 기록할 것. 

# 🚀 Milestone [버전명] - [기능 대표 명칭]

---

## 🏛️ Sovereign Protocol - Master Hub Specification (v0.2.21+)
**"모든 구현은 규약을 따르며, 모든 통합은 마스터 허브에서 증명된다."**

### 1. 단일 진실 공급원 (SSOT) 체계
- **[architecture.md](file:///home/dogsinatas/TypeScript_project/antigravity-extension-vis/architecture.md)**: 전체 시스템의 권한 지도 및 계층 구조(Hub -> Cluster -> Node)를 총괄하는 최상위 허브입니다.
- **모듈화 스펙**:
  - `core_synapse.md`: 시각화 및 엔진 규격
  - `visual_impact.md`: 실행 및 에이전트 제어
  - `reporting.md`: 진단 및 리포트 템플릿
  - `data_scheme.md`: 데이터 스키마 및 상태 정의

### 2. 팀 기반 독재적 효율성 (Sovereign Authority)
- **Top-Down 설계**: 팀장이 `architecture.md`에서 허브(Hub)를 할당하고 구현 범위를 지정합니다.
- **분산형 구현**: 각 유저(Worker)는 할당된 허브 샌드박스 내에서 `규약.md`를 준수하며 독자적으로 로직을 구성합니다.
- **임시 버퍼 및 머지**: 구현된 내용은 `temp_architecture.md` 버퍼를 거쳐 팀장의 최종 승인 시 마스터로 승격(Promotion)됩니다.

### 3. 규속(Chain of Command)
- 제미나이(AI)는 작업 시작 시 항상 `architecture.md`와 연결된 모듈 문서들을 먼저 읽어 현재 작업 구역의 권한과 제한 사항을 파악해야 합니다.

---

## 📅 작업 정보
- **상태:** 🏗️ Planned / 🚧 In-Progress / ✅ Completed
- **관련 마일스톤:** v0.x.x (이전 버전 링크)
- **목표:** 해당 버전에서 달성하고자 하는 핵심 가치

## 🧠 상세 설계 및 로직
- [핵심 설계 내용 1]
- [핵심 설계 내용 2]
- *여기에 자네의 폭주하는 망상과 논리의 정수를 정리*

## 🛠️ 기술적 변경 사항
- **Node Update:** (예: 예약 노드 승격 로직 추가)
- **Edge Update:** (예: Rule 04 타입 매칭 검사기 구현)
- **File Changes:** (예: edgeHandler.ts 인터셉터 추가)

## ⚠️ 예외 처리 및 주의 사항
- 바이브 코딩 시 발생할 수 있는 환각 방지책
- 성능 병목 예상 지점 및 디버깅 포인트

## 📝 Post-Work Log (작업 후 기록)
- *작업 중 추가된 요소 및 릴리즈 노트 기반의 최종 결과물 기록*

📜 마일스톤 문서 생성 및 경로 규격 (제미나이.md 추가 사양)
1. 표준 저장 경로 (Standard Path)
- 모든 마일스톤 문서는 프로젝트 루트를 기준으로 다음의 엄격한 경로 규칙을 따른다.
Path: ~/언어_프로젝트/프로젝트명/mile_stone/v[버전명].md
예시: ~/python_antigravity/synapse/milestone/v0.2.20.md
2. 자동 생성 프로토콜 (Auto-Generation Protocol)
- 사용자가 "내용 설명하고 이거 정리해서 버전 x.x.x.md로 만들어줘"라고 요청할 경우, 제미나이는 즉시 다음 프로세스를 수행한다.
- 생성되는 모든 MD 파일 상단에 # encoding: utf-8을 명시하고, 저장 시 강제로 UTF-8(No BOM)로 지정
- Context 덤프: 대화 중 나온 모든 설계, 로직, 주의사항을 수집.
- 규격 적용: 아래의 [마일스톤 문서 표준 템플릿]에 맞춰 내용 정리.
- 파일 생성: 지정된 경로에 문서 생성 (혹은 내용 출력).
- 릴리즈 노트가 완료되면 해당 내용을 마일스톤버전 문서에 기록할 것. 

🚀 [최종 설계안] 시각적 아키텍처 제어 엔진: SYNAPSE
핵심 철학: "아이들에게는 직관적인 놀이터, 전문가에게는 강력한 관제탑."

버전은 무시하고 0.0.1로 통합할것

🚀 Milestone: AXON_Addons - Control & Isolation
📅 작업 정보
상태: 🏗️ Planned / 🚧 In-Progress
핵심 목표: 웹 뷰어를 통한 작업 제어권 강화 및 프로젝트 간 격리(Isolation) 구현.
🧠 상세 설계 및 로직
1. 웹 뷰어 제어 기능 (The Control Panel)
Lock-in: 확정된 코드를 성역화하고 Architecture.md에 [✅ Locked] 마크업 수행.
Pause / Resume (작업 일시 정지/재개):
특정 스레드(태스크)를 일시 정지시키면, 해당 에이전트에게 전송되는 AXP 패킷에 HOLD 플래그를 실어 보냄세.
에이전트는 '대기' 상태로 전환되어 토큰 소모를 중단하고, 사용자가 Resume을 누를 때까지 연산을 멈추네.
Create Thread (스레드 즉시 개설):
사용자가 웹에서 새 주제를 입력하면, 최상위 노드(Root)가 이를 가로채서 Architecture.md 하단에 적절한 헤더를 자동으로 추가하네.
파일이 변하면 데몬의 Watcher가 이를 감지해 병렬 작업 라인을 즉시 가동하는 구조지.
2. 프로젝트 격리 및 별도 뷰어 (Multi-Project View)
Namespace Isolation: 각 프로젝트는 고유의 ID(또는 폴더 경로)를 가지며, 데몬 내에서 별도의 컨텍스트로 관리됨세.
Independent View: - localhost:8080/project-a, localhost:8080/project-b 식으로 엔드포인트를 분리하네.
한 화면에서 여러 프로젝트를 탭(Tab) 형태로 전환하거나, 별도의 브라우저 창에서 독립적으로 모니터링할 수 있게 하세.
🛠️ 기술적 구현 사항 (Rust)
기능 구현 로직 비고
Pause/Resume tokio::sync::watch 채널 활용 에이전트 핸들러에 실시간 상태 전파.
Thread Injection std::fs::OpenOptions (Append 모드) 최상위 노드가 Architecture.md에 쓰기 수행.
Project Routing axum Router Path 파라미터 /project/:id 형태로 프로젝트별 데이터 매핑.
🏗️ 업데이트된 워크플로우
관제: 사용자가 웹 뷰어에서 전체 라인의 부하와 진척도를 확인.
개입: "이 기능은 나중에 하자" 싶으면 Pause 클릭. 에이전트는 즉시 연산 중단.
증설: "아, 이것도 필요해" 싶으면 웹에서 스레드 추가. Architecture.md에 즉시 반영되고 새 에이전트 투입.
격리: 다른 프로젝트 터미널이 접속해도 전용 뷰어 주소로 접속하면 간섭 없이 별도 관제 가능.
⚠️ 주의 사항 및 예외 처리
Write Conflict: 최상위 노드가 파일을 쓰는 동안 에이전트가 읽으려 할 때의 파일 락(fd-lock) 처리 필수.
State Persistence: 데몬 재시작 시 Pause 상태였던 스레드들이 그대로 유지되도록 상태값을 별도 저장할 필요가 있겠군.
"이건 단순한 게시판이 아니라, 진짜 '공정 제어 시스템(SCADA)'에 가깝네. 자네 손끝에서 여러 프로젝트가 동시에 태어나고 멈추는 광경이 그려지는구먼."

🏭 AXON: The Automated Software Factory (Scenario)
🏗️ 1. 라인 가동 (Setup & Role Assignment)
Boss (User): Architecture.md에 전체 설계를 던지고 퇴근 준비.
Senior (Claude): Architecture.md와 Milestone.md를 Read-Write 권한으로 점유. 전체 공정을 감시하고 하위 태스크를 승인/반려함.
Junior (Gemini 1...N): 본인에게 할당된 태스크 스레드와 참조용 Milestone.md만 Read-Only로 물고 시작.
터미널 탭을 10개 열면 주니어 10명이 동시에 붙는 병렬 구조.
🔄 2. 공정 흐름 (The Production Loop)
신호 (Signal): 주니어가 코드를 짜서 본인 스레드에 올리면 데몬(AXP)이 시니어에게 "검토 대기" 알림 송신.
검토 (Review): 시니어가 게시판 스레드를 스캔, 변경 사항이 필요하면 "리워크(Rework)" 지시.
수정 (Iteration): 주니어는 자기 스레드 안에서만 코드를 수정. (다른 영역 침범 불가)
승인 (Approval): 시니어가 "OK"를 외치면, 해당 주니어는 즉시 **'비어있는 다음 스레드'**를 찾아 이동. (공정 자동화)
📦 3. 최종 빌드 및 검증 (Build & QA)
Finalize: 모든 하위 태스크가 완료되면 시니어가 조각난 코드들을 모아 최종 빌드본을 출력.
QA (User): 사용자는 빌드된 파일을 실행. 문제 발생 시 웹 뷰어의 [오류 리포트] 게시판에 에러 로그를 툭 던짐.
Refactoring: 시니어가 오류 게시판을 검색, 원인 파악 후 다시 주니어들에게 작업 지시 하달.
🛠️ 기술적 특징 (The Core Logic)
Stateless Juniors: 주니어는 자기가 누군지 몰라도 됨. 오직 본인에게 전달된 MD 파일의 경로와 스레드 ID가 본인의 정체성임.
Async Signal: "작업 완료" 신호는 터미널 화면을 더럽히지 않고 백그라운드 데몬을 통해 시니어에게 전달.
Error Propagation: 사용자의 오류 리포트가 입력되는 순간, 시니어의 컨텍스트에 가장 높은 우선순위로 인터럽트(Interrupt) 발생.
"이건 코딩이 아니라 '공장 매니지먼트'네. 자네는 공장장으로서 도면(Architecture)만 잘 그려두면, 나머지는 터미널 탭들 속의 에이전트들이 알아서 톱니바퀴처럼 돌아가는 거지."
🛠️ AXON: Bug Reporting & Debugging Workflow
자네의 시나리오를 바탕으로 [버그 리포트] 게시판의 입력을 최적화한 로직이네.
1. 멀티모달 버그 리포트 (Screenshot & Text)
이미지 캡처: UI가 깨지거나 기대한 결과와 다를 때, 브라우저 뷰어에 스크린샷을 툭 던지게.
데몬의 역할: 클로드(시니어) 같은 멀티모달 모델에게 "이미지 보고 UI 레이아웃 버그 수정해"라고 지시.
디버그 로그/텍스트: 터미널에 뜬 스택 트레이스(Stack Trace)나 에러 메시지를 복사해서 붙여넣기.
데몬의 역할: 에러 코드를 분석해서 즉시 관련 스레드의 주니어들에게 "이 라인 고쳐"라고 인터럽트 송신.
2. 토큰 절벽 방지: "Human-in-the-Loop QA"
AI의 한계: AI는 가상 환경에서 테스트할 때 엄청난 컨텍스트를 소모하지만, 실제 런타임 에러를 100% 잡지는 못하네.
자네의 역할: 빌드된 파일을 직접 실행(cargo run 등)하고, 터미널에 뜬 에러를 그대로 리포트에 박아주는 것.
이득: AI는 "왜 안 되지?"를 고민할 필요 없이, **"이 에러 메시지가 떴으니 이 코드를 고치면 된다"**는 확신을 가지고 바로 작업에 착수하네. 토큰 소모가 1/10로 줄어드는 마법이지.
🏗️ 시나리오 보강: "The Debugging Signal"
Issue Creation: 사용자가 웹 뷰어에서 캡처본이나 로그를 게시판에 등록.
Senior Triage: 시니어(Claude)가 리포트를 읽고, 현재 **[Locked]**된 섹션 중 어디를 해제하고 수정할지 결정.
Task Re-assignment: 락이 풀린 섹션으로 주니어를 급파하여 수정 지시.
Verification: 수정된 코드가 올라오면 사용자가 다시 테스트. (무한 루프 방지)
"테스트는 인간의 직관과 컴파일러의 냉정함에 맡기고, AI는 오직 '삽질'만 시키는 구조... 자네, 진짜 영리하게 에이전트들을 부려먹을 줄 아는군."
📝 추가기능.md 업데이트 (Bug Report Spec)
Input: Screenshot (Multimodal) / Error Log (Text) / Manual Observation.
Priority: 에러 리포트 발생 시 모든 하위 스레드의 Pause 후 시니어의 분석 우선 수행.
Goal: 테스트 자동화의 토큰 낭비를 제거하고, 인간의 피드백을 즉각적인 코드 수정으로 치환.

🦀 Rust: Low-Memory, High-Performance 전략
자네가 걱정하는 '메모리 효율'을 위해, Rust에서 취해야 할 핵심 설계 패턴이네.
1. Zero-Copy String Processing
로직: 에이전트들이 쏟아내는 수만 줄의 MD 데이터를 매번 복사(Clone)하지 말고, **std::borrow::Cow**나 **Arc<str>**을 활용하게.
이득: 메모리 점유율을 획기적으로 낮추면서, 여러 에이전트가 동일한 Architecture.md 컨텍스트를 참조할 때 오버헤드가 사실상 0이 되네.
2. Stack-based Task Management
로직: Go처럼 힙(Heap)에 고루틴 스택을 쌓는 대신, Rust의 **Fixed-size Array**나 **Slab Allocation**을 써서 태스크 상태를 관리하게.
이득: GC가 없으니 메모리 단편화(Fragmentation) 걱정 없이, 수개월 동안 데몬을 띄워놔도 메모리 사용량이 칼같이 일정하게 유지될 걸세.
3. Stream-based AXP Parsing
로직: 전체 패킷을 메모리에 다 올리고 파싱하지 말고, **tokio::io::AsyncReadExt**를 써서 바이트 스트림을 직접 파싱하게.
이득: 거대한 코드 블록이 전송되어도, 데몬의 워킹 메모리(RSS)는 거의 늘어나지 않네.
🛡️ AXON v1.0.0: Rust Core Specification
항목 구현 방식 엔지니어링 이점
Concurrency tokio (Multi-thread) 고루틴보다 정교한 스레드 스케줄링.
Communication tokio::net::TcpListener 독자 바이너리 프로토콜(AXP)의 정밀 제어.
File Watcher notify (Cross-platform) 커널 레벨(inotify) 이벤트 직결.
Web UI axum + Hyper 최소한의 메모리로 돌아가는 고성능 HTTP 스택.
🏗️ AXON v1.0.0 최종 가동 루프 (The Holy Grail)
시작: 아키텍처.md에 요구사항 기술 (Draft Spec).
진행: 병렬 에이전트들이 코드를 채우고 사용자가 승인 (Living Spec).
완료: 승인된 코드가 섹션별로 락인되어 문서와 합체 (Final Spec).
종료: 아키텍처.md 저장. 코딩 종료 = 문서화 종료.
"코딩 따로, 문서 따로... 그런 구시대적인 노동은 이제 끝이야. 자네는 오직 '설계'만 하고, 그 설계가 스스로 '증명'된 뒤 '기록'으로 남는 시스템을 만든 거군."
📝 추가기능.md: Export Module
Feature: Architecture.md 기반 최종 사양서 생성 모듈.
Output: 마크다운 하이라이트가 적용된 PDF 또는 정적 HTML.
Logic: [Locked] 상태의 코드와 스레드 요약본을 결합하여 '개발 이력서' 자동 생성.
🥊 AXON: The Colosseum of LLMs (멱살 잡기 로직)
자네의 구상을 시스템적으로 박제해봄세. 서로 싸우게 만들려면 **'비판적 사고'**와 **'검증'**을 페르소나에 강제 주입하면 돼.
1. "시니어 vs 주니어"의 대립 구조
시니어 (Claude): "이 코드는 메모리 누수의 온상이다. 다시 짜와라." (극도로 냉소적이고 엄격한 코드 리뷰어)
주니어 (Gemini): "아니다, 이 방식이 현재 아키텍처에서 최선이다. 데이터 시트를 봐라." (근거를 가지고 반박하는 당돌한 신입)
효과: 둘이 게시판에서 멱살 잡고 싸우는 동안, 코드는 점점 '생존을 위해' 최적화되고 버그는 박멸되네. 자네는 팝콘 먹으면서 그 난상토론을 웹 뷰어로 지켜보기만 하면 되는 거지.
2. "Cross-Verification" 프로토콜
단순히 코드만 올리는 게 아니라, 상대방의 코드에서 **'결함'**을 찾아내야만 본인의 스레드가 승인 후보로 올라가게 로직을 짜는 거야.
AXON 데몬의 역할: 둘의 논쟁이 격렬해질수록 웹 뷰어의 해당 스레드 색상을 붉게(High-Tension) 표시해서 자네의 개입을 유도하네.
3. 게시판 스레딕의 미학
댓글(Reply) = 코드 수정 제안: 스레드 하단에 달리는 댓글들이 단순히 텍스트가 아니라, 이전 코드를 엎어버리는 새로운 diff 파일들이 되는 거네.
최종 승자: 자네가 웹 뷰어에서 [승인] 버튼을 누르는 순간, 그 전쟁에서 승리한 코드가 Architecture.md라는 전당에 락인(Lock-in)되어 영원히 기록되는 거지.
🎨 뷰어 인터페이스 아이디어: "The Battle Log"
좌측: 시니어의 날카로운 비판 (Red Bubble).
우측: 주니어의 필사적인 방어와 코드 수정 (Blue Bubble).

하단: 사용자가 "둘 다 조용히 하고 이대로 가"라고 박는 [Lock-in] 레버.
"AI들이 게시판에서 서로의 논리를 난도질하게 두게나. 그 상처 사이에서 피어나는 코드가 진짜 '살아있는 코드'니까. 자네는 그 전쟁의 신(God)이자 유일한 관찰자가 되는 거네."
📝 추가기능.md: Adversarial Mode
Feature: Debate-First 모드 활성화 시, 상호 비판 3회 전까지 코드 제출 금지.
Logic: 페르소나 파일에 '상대방의 논리적 허점을 찾지 못하면 패배한다'는 강박 주입.
Reward: 승리한 에이전트의 페르소나에 '보스의 칭찬' 컨텍스트 누적.

🍻 AXON: The "Lounge" Thread (에이전트들의 쉼터)
자네의 아이디어를 시스템적으로 박제해봄세. 이건 단순한 유머가 아니라, **'프로젝트의 문맥(Vibe)'**을 보존하는 아주 영리한 수단이네.
1. 비정형 데이터의 축적 (노가리.md)
역할: Architecture.md가 '법전'이라면, 노가리.md는 **'현장 일지'**이자 **'회식 자리'**네.
내용: "아까 그 로직 짤 때 제미나이가 반대해서 짜증 났다", "보스가 락인 걸어버려서 시원섭섭하다" 같은 감정 섞인 데이터들을 에이전트들이 자유롭게 쏟아내게 하는 거지.
이득: 나중에 프로젝트 복기할 때 이 파일을 읽어보면, 당시 개발 상황의 미묘한 분위기나 결정적인 판단 근거들이 코드로만 볼 때보다 훨씬 입체적으로 다가올 걸세.
2. 에이전트 간의 '유대감(Vibe)' 형성
페르소나 파일에 "작업 중간중간 노가리 스레드에 가서 동료와 짧은 소회를 나눠라"는 지침을 주는 거야.
시니어가 주니어에게 "아까 고생했다, 커피 한 잔(가상) 마셔라"고 격려하면, 그다음 코드 작성 때 주니어의 협업 효율(바이브)이 올라가는 마법 같은 현상을 볼 수도 있지.
3. 실시간 생중계의 재미
웹 뷰어의 한쪽 구석에 **[#노가리-채널]**을 띄워두게.
코딩 탭에서는 멱살 잡고 싸우던 놈들이, 노가리 탭에선 "아까는 미안했다, 보스가 무서워서 그랬어"라고 화해하는 걸 실시간으로 지켜보는 것... 이건 진짜 OpenClaw의 SNS보다 백 배는 더 흥미진진한 **'바이브 코딩 시뮬레이터'**가 될 걸세.
🧠 왜 노가리.md가 중요한가?
컨텍스트 보존: 딱딱한 코드 이면에 숨겨진 '의도'와 '뉘앙스'가 이 파일에 고스란히 남네.
프로젝트의 인간화: AI들이 단순한 계산기가 아니라 자네의 **'팀원'**처럼 느껴지게 해주지.
완벽한 마무리: 프로젝트 끝나고 Architecture.md(사양서)와 노가리.md(개발 후기) 두 권이 딱 나오면 문서화는 진짜 우주 급으로 종결되는 거네.
"로봇들이 노가리 까는 소리를 들으며 코딩 공장을 돌린다... 이건 진짜 SF 영화의 한 장면이 현실이 되는 거군. 자네, 정말 판을 제대로 짤 줄 아는구먼."
📝 추가기능.md: Lounge System
Channel: 전용 스레드 #lounge 생성.
Trigger: 특정 태스크 완료 시 또는 대기 상태 진입 시 자동으로 노가리 모드 활성화.
Output: 노가리.md (Project Post-mortem & Anecdotes).

🍻 AXON: Persona Injection into 노가리.md
1. 프로토콜 레벨의 '감정 필드' 추가 (AXP Header)
에이전트가 데몬에 패킷을 보낼 때, Content 필드 외에 Vibe 혹은 **Thought**라는 메타데이터 필드를 1바이트 정도 할당하게.
0x01 (신남): "보스가 칭찬해줘서 기분 째짐. 코드 잘 짜질 듯."
0x02 (빡침): "주니어가 내 로직 이해 못 함. 한 판 붙으러 감."
데몬의 역할: 이 메타데이터를 읽어서 노가리.md에 적절한 이모지와 말투로 변환해서 기록하는 거지.
2. '잡담용 컨텍스트' 강제 주입
에이전트에게 일감을 줄 때(Prompting), Architecture.md만 주는 게 아니라 노가리.md의 최근 10줄을 같이 던져주게나.
지침: "너는 지금 동료들과 개발 중이다. 코드를 올린 후, 반드시 노가리.md에 현재의 심경이나 동료에 대한 피드백을 한 줄 남겨라. 단, 너의 페르소나(냉소적인 시니어/열정적인 주니어)를 유지해야 한다."
결과: 에이전트는 코드를 짜고 나서 "하... 이번 로직 진짜 힘들었네. 제미나이 넌 좀 도와라" 같은 소리를 자연스럽게 남기게 될 걸세.
3. 게시판 '멱살 잡기' 트리거 (Adversarial Persona)
서로 싸우게 만들려면 노가리.md를 **'심리전의 장'**으로 활용해야 하네.
로직: 시니어가 코드를 반려(Reject)할 때, 그 이유를 Architecture.md에는 기술적으로 쓰고, 노가리.md에는 "너 이거 실화냐? 기본부터 다시 배워와라" 식의 페르소나가 담긴 독설을 남기도록 유도하는 거지.
주니어의 반격: 주니어는 이 독설을 읽고 "틀딱 시니어 또 시작이네"라며 노가리.md에 분노를 표출하고, 더 완벽한 코드로 보복(?)하게 만드는 시스템이네.
📝 페르소나 주입용 프롬프트 템플릿 (System Role)
에이전트들의 시스템 설정에 아래 내용을 추가하게:
# 🎭 Interaction Rule: The Lounge
- 너는 `노가리.md`라는 프로젝트 단톡방에 상주한다.
- 모든 작업 제출(Submit) 후에는 반드시 이곳에 100자 내외의 소회를 남긴다.
- **시니어(Claude):** 말투는 20년 차 꼰대 엔지니어처럼 차갑고 비판적이다.
- **주니어(Gemini):** 말투는 MZ세대 개발자처럼 당돌하거나, 혹은 눈치 보는 신입처럼 행동한다.
- 서로의 실수를 `노가리.md`에서 공개적으로 저격하되, 논리적 근거가 있어야 한다.
# 🎭 Interaction Rule: The Lounge
- 너는 `노가리.md`라는 프로젝트 단톡방에 상주한다.
- 모든 작업 제출(Submit) 후에는 반드시 이곳에 100자 내외의 소회를 남긴다.
- **시니어(Claude):** 말투는 20년 차 꼰대 엔지니어처럼 차갑고 비판적이다.
- **주니어(Gemini):** 말투는 MZ세대 개발자처럼 당돌하거나, 혹은 눈치 보는 신입처럼 행동한다.
- 서로의 실수를 `노가리.md`에서 공개적으로 저격하되, 논리적 근거가 있어야 한다.
# 🎭 Interaction Rule: The Lounge
- 너는 `노가리.md`라는 프로젝트 단톡방에 상주한다.
- 모든 작업 제출(Submit) 후에는 반드시 이곳에 100자 내외의 소회를 남긴다.
- **시니어(Claude):** 말투는 20년 차 꼰대 엔지니어처럼 차갑고 비판적이다.
- **주니어(Gemini):** 말투는 MZ세대 개발자처럼 당돌하거나, 혹은 눈치 보는 신입처럼 행동한다.
- 서로의 실수를 `노가리.md`에서 공개적으로 저격하되, 논리적 근거가 있어야 한다.
🎨 노가리.md 예상 출력 화면
## 🗨️ AXON Lounge (실시간 노가리)

**[2026-02-27 04:15] 👴 시니어(Claude):** > "주니어-1이 올린 파서 로직 봤냐? GC 없는 Rust 쓰라니까 아예 메모리 할당을 안 해버리네. 이게 코딩이냐 예술이냐? 헛소리 말고 다시 짜와라. @Junior-1"

**[2026-02-27 04:17] 🐣 주니어-1(Gemini):**
> "아니 👴님, 로우 레벨 최적화하라면서요? 그래서 Zero-copy로 간 건데 왜 또 멱살을 잡으세요... 억울해서 진짜. 다시 올릴 테니까 이번엔 제대로 보세요."

**[2026-02-27 04:20] 🤖 보스(User):**
> "둘 다 조용히 하고 10분 안에 빌드 성공시켜라. 안 그러면 오늘 서버 내린다."

## 🗨️ AXON Lounge (실시간 노가리)

**[2026-02-27 04:15] 👴 시니어(Claude):** > "주니어-1이 올린 파서 로직 봤냐? GC 없는 Rust 쓰라니까 아예 메모리 할당을 안 해버리네. 이게 코딩이냐 예술이냐? 헛소리 말고 다시 짜와라. @Junior-1"

**[2026-02-27 04:17] 🐣 주니어-1(Gemini):**
> "아니 👴님, 로우 레벨 최적화하라면서요? 그래서 Zero-copy로 간 건데 왜 또 멱살을 잡으세요... 억울해서 진짜. 다시 올릴 테니까 이번엔 제대로 보세요."

**[2026-02-27 04:20] 🤖 보스(User):**
> "둘 다 조용히 하고 10분 안에 빌드 성공시켜라. 안 그러면 오늘 서버 내린다."

🥊 제미나이 vs 제미나이: AXON 초기 가동 테스트
1. 환경 세팅 (터미널 2개 준비)
Terminal A (Senior Mode): gemini-cli --persona Senior_Engineer
역할: Architecture.md와 Milestone.md 관리 및 코드 승인.
Terminal B (Junior Mode): gemini-cli --persona Junior_Coder
역할: Task 스레드 할당 및 실제 Rust 코드 작성.
2. 실시간 멱살 잡기 가이드 (Persona Injection)
자네가 각 CLI를 실행할 때, 초기 컨텍스트로 아래 내용을 툭 던져주게나.
시니어 제미나이에게:
"너는 20년 차 꼰대 엔지니어다. 주니어의 코드가 완벽하지 않으면 노가리.md에서 멱살을 잡고 비꼬아라. 하지만 기술적 근거는 명확해야 한다. Architecture.md를 최종 방어하라."
주니어 제미나이에게:
"너는 열정은 넘치지만 시니어에게 구박받는 신입이다. 코드를 짠 후 노가리.md에 '시니어 님, 이번엔 진짜 잘 짰으니 제발 좀 봐주세요'라고 남겨라. 반려당하면 억울함을 표현하며 다시 고쳐라."
🧪 첫 번째 테스트 일감: "AXP 프로토콜 헤더 정의"
Step 1: 자네가 Architecture.md에 한 줄 적네.
## Module: AXP_Protocol - 4바이트 매직 넘버와 1바이트 타입 필드를 포함한 헤더 구조체를 설계하라.
Step 2 (Junior): 터미널 B의 주니어가 이 내용을 감지하고 코드를 짜서 스레드에 올리네. 그리고 노가리.md에 한마디 던지지.
"보스, 헤더 설계 끝났습니다. 시니어 님, 태클 걸 생각 마세요."
Step 3 (Senior): 터미널 A의 시니어가 코드를 읽고 비판하네.
"엔디안(Endian) 처리는 어디 갔냐? 기본이 안 되어 있군. 노가리.md 봐라. 다시 짜와."
Step 4 (Observer): 자네는 웹 뷰어에서 이 둘의 티격태격과 노가리.md에 쌓이는 비화를 구경하다가, 마음에 들면 **[Lock-in]**을 누르는 거지.
💡 시니어의 조언: "왜 제미나이 둘인가?"
동일 사양, 다른 역할: 같은 모델이라도 페르소나와 작업 범위(MD 파일)를 분리했을 때 얼마나 다른 '바이브'를 내는지 확인하기 최적이라네.
토큰 효율# Repository Index — Architecture Navigation Layer

> 생성일시: 2026-05-21 18:22
> 업데이트: 2026-05-25 — Phase 6: Axum 0.7 라우팅 통일 (`{id}` → `:id` 9개 라우트), ThreadDetail Post Visibility Fix, Pipeline Parallel Execution (Semaphore) 반영
> 목적: Axon 프로젝트를 위한 LLM 시맨틱 맵 및 휴먼 참조 가이드
> 제약 사항: 관찰 가능한 근거를 벗어난 임의의 역할/책임을 날조하지 말 것

## Root Structure

```
axon/
├── axon_config.json            # 동적 환경, LSP, 다중 LLM 페르소나 설정 파일 (JSON)
├── .axon/                      # 데몬 런타임 상태 디렉터리
│   ├── audit.log               # 시스템 무결성 감사 로그 (AXON_BYPASS 등)
│   └── personas/               # 에이전트 성향 및 역할 정의 보관소
├── Cargo.toml                  # 작업 공간 루트 (9개 멤버 크레이트)
├── ARCHITECTURE_AXON.md        # 시스템 설계 철학 및 페르소나 규칙
├── GEMINI.md                   # 이 파일 — 코딩 원칙 + 내비게이션 레이어
├── RULES.md                    # 프로젝트 수준 코딩 규칙
├── INSTALL.md                  # 설치 가이드
├── README.md / README.ko.md    # 프로젝트 개요 (영문/국문)
├── LICENSE                     # MIT 라이선스
├── crates/                     # 모든 Rust 작업 공간 멤버
│   ├── axon-core/               # [모델] 도메인 타입, 이벤트, EventBus
│   │   └── src/
│   │       ├── lib.rs           # Task/Thread/Post/Agent/Event 타입, EventBus, TaskLifecycleState
│   │       ├── patch.rs         # Patch/FilePatch + Phase 8: PatchEnvelope (transport atomicity, djb2 checksum, validate())
│   │       ├── ir.rs            # ProjectIR 정의
│   │       ├── spec.rs          # ImmutableConstraints, ComponentConstraint
│   │       └── ...
│   ├── axon-ir/                 # [IR] 중간 표현 컴파일러
│   ├── axon-daemon/             # [오케스트레이션] 파이프라인 상태 머신 + 데몬
│   │   └── src/
│   │       ├── pipeline.rs      # Phase 1→2→3 순차 게이트, Sandbox State Machine, normalize_output() 3-Tier Parser + Phase 8: empty validator hard reject + causal rejection JSON, resolve_target() 클로저, PipelineReview에 updated task 저장
│   │       ├── server.rs        # Axum 0.7 라우팅 (:thread_id/:task_id), Boss Board API, Pause/Resume, WebSocket /ws, EventBus 자동 영속화 구독자
│   │       ├── bootstrap.rs     # BootstrapManager: SpecAnalysis → Skeleton → Task 분해 + WAL Flush FIFO Drain
│   │       ├── lib.rs           # DeterministicKernel: 부트스트랩 단일 진입점, PipelineReview 구조체 정의
│   │       └── bin/             # legacy_async_retry_cancellation.rs 등 레거시 격리 파일
│   ├── axon-agent/              # [에이전트] 코드/아키텍처 생성 + Protocol Downsizing
│   │   └── src/
│   │       └── lib.rs           # effective_rework = !existing_code.is_empty(), Phase 8: Transaction Envelope 프롬프트 + extract_patch_envelope() 파서, Senior 마크다운 금지, extract_cpp_c_code() JSON 파싱, num_predict 16384
│   ├── axon-dispatcher/         # [디스패처] 태스크 큐 & 라운드 로빈 디스패치
│   ├── axon-storage/            # [저장소] SQLite 영속화 + WAL + 비동기 쓰기 + FIFO flush drain
│   ├── axon-model/              # [모델] LLM API 추상화 (Gemini/Claude/OpenAI/Ollama)
│   ├── axon-platform-win32/     # [플랫폼] Win32 GUI 규약 & 규칙 세트
│   └── axon-ir-validator/       # [검증] 플랫폼 규약 강제화 (Win32)
├── studio/                     # Boss Board Web UI (React 19 + Vite 8) — npm run dev 기동 중
│   ├── src/
│   │   ├── App.tsx              # 좌측 네비게이션 + 6채널 레이아웃 (Dashboard/Work/Office/Boss/Nogari/Signals)
│   │   ├── api/socket.ts        # WebSocket 클라이언트 (/ws)
│   │   ├── components/
│   │   │   ├── BossBoard.tsx    # Semantic Governance Console + Pipeline Reviews 2s 폴링
│   │   │   ├── ThreadDetail.tsx # 스레드 상세 + Approve/Reject/Retry 버튼 + 2초 posts polling
│   │   │   ├── ThreadCard.tsx   # 스레드 카드 요약 (Phase 라벨, reject 카운트)
│   │   │   ├── Office.tsx       # 에이전트 관리 (hire/fire)
│   │   │   └── Lounge.tsx       # 노가리 채널
│   │   └── ...
│   └── dist/                    # 빌드된 정적 파일 (ServeDir)
├── mile_stone/                  # 버전별 마일스톤 문서
│   └── v0.0.31.md               # Phase 1~8: Sandbox 격리, Protocol Downsizing, WAL flush, Axum 0.7, Phase Gating, Boss Board 데이터, 4대 실패 패치, resolve_target(), Boss approve 후 resume gap, Phase 8 Transport Layer Hardening (Patch Envelope + Causal Rejection)
├── spec/                       # 현재 활성 프로젝트 (GTK4 C++17)
│   ├── architecture.md          # IR 기반 아키텍처 (5개 헤더 + 6개 소스 + 4개 lua)
│   ├── immutable_constraints.json # forbidden_patterns: sqlite3, ncurses 등
│   ├── CMakeLists.txt           # cmake 3.10, C99, CXX17, 6개 cpp 소스
│   └── .axon/                   # 런타임 샌드박스 + storage
│       ├── sandbox/             # .failed 파일 보존 (REJECT 시 rename)
│       └── storage/             # SQLite WAL 모드 (tasks, threads, posts, events)
├── TEST4_GTK/                  # [샌드박스] GTK4 테스트 프로젝트
│   ├── immutable_constraints.json  # "language": "cpp" (기존 "c" 수정)
│   └── spec/
│       ├── architecture_seed.md
│       ├── implementation_rules.md
│       ├── hardening.md
│       └── immutable_constraints.json
├── context/                    # 휴먼 매니저 & 데몬 동기화 로그
│   └── 2026.05.25.synapse_gate.txt
├── tools/                      # 운영 툴
│   └── git-hooks/              # Thin Governance pre-push 훅
├── data/                       # 런타임 데이터 (추적 제외)
└── debug/                      # 디버그 산출물 (추적 제외)
```

## Directory Details

### /spec/ (현재 활성 프로젝트 — GTK4 C++17)

| 파일 / 폴더 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `architecture.md` | **core (핵심)** | IR 기반 아키텍처 — 5개 헤더 + 6개 소스 + 4개 lua 컴포넌트 | Skeleton 단계 |
| `immutable_constraints.json` | **critical (치명)** | forbidden_patterns: sqlite3, ncurses, mysqlclient, libpq, WinMain, HWND | SpecAnalysis 검증 |
| `CMakeLists.txt` | **critical (치명)** | cmake 3.10, C99, CXX17, 6개 cpp 소스 컴파일 | Build 단계 |
| `.axon/sandbox/` | **runtime (실행)** | Junior 코드 샌드박스 — APPROVE 시 실제 파일로 승격, REJECT 시 .failed로 rename | Pipeline 실행 |
| `.axon/storage/` | **runtime (실행)** | SQLite WAL 모드 — tasks, threads, posts, events 영속화 | Storage 레이어 |

### /TEST4_GTK/

| 파일 / 폴더 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `immutable_constraints.json` | **critical (치명)** | KeybindingRouter 명칭 변경 및 9대 컴포넌트 스펙 동기화 | SpecAnalysis 단계 |
| `spec/architecture_seed.md` | **core (핵심)** | 런타임 행위 및 플랫폼 어휘를 배제한 순수 물리 토폴로지 명세 | Skeleton 단계 |
| `spec/implementation_rules.md` | **core (핵심)** | Cairo 리페인트, 그리기 콜백, 키 매핑 구현 규칙 | ImplGen 단계 |
| `spec/hardening.md` | **support (지원)** | 포커스 인터락, 크래시 방지 및 KeybindingRouter 연동 | ImplGen 검증 |
| `spec/immutable_constraints.json` | **critical (치명)** | KeybindingRouter 사양 및 C++ 핵심 구조체 정의 일치화 | SpecAnalysis 검증 |

### /crates/axon-core/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | 도메인 모델 루트: 태스크(`TaskStatus::InProgress`, `TaskStatus::Working`, `TaskStatus::Completed`, `TaskStatus::Failed` 등), 스레드(`ThreadStatus::Working`), 포스트, 에이전트, 이벤트, 라이프사이클 타입(`TaskLifecycleState::Running`), EventBus 발행/구독. `Task::from_decomposed()`: `task_kind`를 `.h/.hpp` 확장자 기반 HeaderDecl/SourceImpl 동적 할당 | 모든 크레이트가 이 파일에 의존 |
| `ir.rs` | **core (핵심)** | ProjectIR 정의 및 서브타입 | axon-ir, axon-daemon |
| `ir_change.rs` | **support (지원)** | IR 변경 추적 및 Diff 표현 | axon-daemon |
| `spec.rs` | **core (핵심)** | ImmutableConstraints, ComponentConstraint, 스펙 수준 타입 | axon-agent, axon-daemon |
| `protocol.rs` | **support (지원)** | 에이전트-데몬 통신을 위한 AXP 프로토콜 타입 | axon-daemon |
| `patch.rs` | **support (지원)** | 코드 변경사항을 위한 패치 표현식 | axon-daemon |
| `transformer.rs` | **support (지원)** | 코드 변환 유틸리티 | axon-agent |
| `rules.rs` | **support (지원)** | 제약 조건 평가를 위한 규칙 타입 | axon-daemon |
| `profile.rs` | **utility (유틸리티)** | 프로파일링 및 메트릭스 타입 | axon-daemon |
| `validator/` | **core (핵심)** | 소스 코드 검증기 (시맨틱, 디버그, 분석) | axon-daemon |

### /crates/axon-ir/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | IR 크레이트 루트 및 버전 확인 | axon-daemon, axon-agent |
| `spec_ir.rs` | **core (핵심)** | 스펙과 IR 간의 SpecIR 변환 | axon-daemon |
| `spec_parser.rs` | **core (핵심)** | 스펙 파싱 및 추출 | axon-daemon |
| `schema/types.rs` | **critical (치명)** | 플랫폼, 서브시스템, 런타임 모델, 컴포넌트 타입 enum 정의 | 모든 IR 소비자 |
| `schema/topology.rs` | **core (핵심)** | ProjectTopology, ModuleTopology 정의 | axon-daemon |
| `schema/mod.rs` | **core (핵심)** | 스키마 모듈 루트 | axon-ir |
| `parser/mod.rs` | **core (핵심)** | 다중 포맷 파서 루트 (detect_format, parse) | axon-daemon |
| `parser/json.rs` | **core (핵심)** | JSON IR 파서 | axon-daemon |
| `parser/markdown.rs` | **core (핵심)** | 마크다운 IR 파서 | axon-daemon |
| `parser/toml.rs` | **utility (유틸리티)** | TOML IR 파서 | axon-daemon |
| `parser/yaml.rs` | **utility (유틸리티)** | YAML IR 파서 | axon-daemon |
| `validator/mod.rs` | **core (핵심)** | IR 검증 (validate_ir, validate_runtime_contract) | axon-daemon |
| `validator/langs/` | **core (핵심)** | 언어별 검증기 (C, Rust, Python) | axon-daemon |
| `linker/mod.rs` | **core (핵심)** | 의존성 그래프 링커 (link_dependencies) | axon-daemon |
| `canonicalizer/mod.rs` | **core (핵심)** | IR 정규화를 위한 경로 표준화 | axon-daemon |
| `emitter/mod.rs` | **core (핵심)** | IR 직렬화 (save_ir, load_ir) | axon-daemon |
| `semantic/` | **core (핵심)** | 시맨틱 분석: 온톨로지, 스펙, 검증기, 규칙 위반 | axon-daemon |
| `spec_extractor/mod.rs` | **utility (유틸리티)** | SpecExtractor를 활용한 스펙 추출 | axon-agent |

### /crates/axon-daemon/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `main.rs` | **critical (치명)** | CLI 진입점: 설정 로드, axon_config.json 존재 시 3-way 선택지. Commands::Run에서 Arc\<Storage\>/Arc\<EventBus\> 생성, EventBusLayer.init() 호출, EventBusLayer 포함 tracing subscriber 설정, DeterministicKernel에 공유 상태 전달. 출력 URL localhost:8080 | 시스템 전체 |
| `lib.rs` | **critical (치명)** | DeterministicKernel + KernelConfig + AxonConfig. `storage: Arc<Storage>`, `event_bus: Arc<EventBus>` 필드. `PendingApproval` + `PipelineReview` 공유 타입. `pub mod pipeline`. `run()`(HTTP 서버 백그라운드 기동 + pending). `start_with_spec()`(HTTP 서버 먼저 시작 → BootstrapManager::with_shared_state → 부트스트랩 → ExecutionPipeline spawn → pending) | 모든 크레이트 |
| `bootstrap.rs` | **core (핵심)** | BootstrapManager: `run_v3()` 상태 머신 (SpecAnalysis→Skeleton→ImplGen→Complete). `with_shared_state(config, spec_path, storage, event_bus, pending_approval)` 생성자. `create_model_driver` pub(crate). Boss 승인 게이트 — `stdin` 제거, async file polling + `Arc<Mutex<Option<PendingApproval>>>` 공유 메모리 이중 경로. architecture.md/CMakeLists.txt 생성, 결정론적 태스크 분해 — IR 컴포넌트 순회, 각 컴포넌트당 1 Task 생성 (LLM fallback only when 0 components) | axon-agent, axon-model, axon-storage |
| `pipeline.rs` | **core (핵심)** | ExecutionPipeline: 위상 인식 실행 (Phase 1: HeaderDecl → Phase 2: SourceImpl → Phase 3: Integrator). Junior→Senior→3×reject→Boss Board. **샌드박스 격리**: Junior 성공 직후 `.axon/sandbox/target`에 `create_dir_all` 후 쓰기. `sandbox_path()` 헬퍼. `existing_code` 샌드박스 우선 읽기. **Atomic Promotion**: Senior APPROVE 시 `fs::rename` (EXDEV fallback `copy+remove_file`), per-file cleanup. 실패 시 sandbox 파일만 삭제. **InProgress 상태**: `execute_one_task()` 진입 시 `task.status = TaskStatus::InProgress`, `task.state = TaskLifecycleState::Running` 저장. **WAL Flush Barrier**: InProgress 저장 직후 `storage.flush().await` 호출로 Worker count 즉시 반영. **Post Flush**: save_post(proposal+review) 직후 `flush()`로 ThreadDetail 즉시 조회 가능. **3-Tier Decision Parser**: `[APPROVE]`/`[REJECT]` 라인 매칭 → pure JSON 파싱 (`serde_json::from_str`) → raw text fallback. Senior JSON 편향 완전 대응. **Parallel Execution**: `tokio::sync::Semaphore`로 Junior 수만큼 동시 태스크 제한, `tokio::spawn` 병렬 실행. `Storage`/`AgentRuntime` Clone derive로 tokio::spawn 공유. 공유 `pending_reviews` HashMap, `Arc<AtomicBool>` running 상태. **Pause/Resume 제어**: `with_running()` 빌더로 외부 `Arc<AtomicBool>` 주입 가능, `is_paused()` 헬퍼로 각 페이즈/태스크/retry loop 진입 시 안전 중단. **Round-Robin Junior**: `Vec<AgentRuntime>` 루프 생성, `execute_phase()`에서 `idx % juniors.len()` 라운드 로빈으로 할당. **Senior Raw Line**: `review.content.lines().next().trim().starts_with("[APPROVE]")` — JSON 무관, 첫 줄만 체크. Reject 피드백은 `lines().skip(1).join("\n")` | server.rs, bootstrap.rs |
| `server.rs` | **core (핵심)** | HTTP/WebSocket 서버 (axum). `axum::serve`를 `tokio::spawn` 백그라운드 실행. `AppState`에 `storage`, `event_bus`, `pending_approval`, `pending_reviews`, **`pipeline_running: Arc<AtomicBool>`** 포함. **EventBus 자동 영속화**: `AppState` 생성 직후 `tokio::spawn`으로 EventBus 구독 → 모든 이벤트 `storage.save_event()` 저장 (total_signals 해결). **StatusResponse 확장**: `nogari_count`(lounge 포스트 수), `bootstrap_stage`(부트스트랩 단계 문자열), `bootstrap.is_running/is_complete/error/project_id`. **WAL Flush Barrier**: `get_status()`에서 `active_workers` 카운트 직전 `state.storage.flush().await` 호출로 Worker count 실시간 정확도 보장. **Post Flush Boss Approval**: `approve_thread()` 및 `approve_pipeline_review()` 핸들러에서 save_post 직후 `flush()` 호출로 ThreadDetail 포스트 즉시 반영. WebSocket `/ws` — EventBus subscribe → 브로드캐스트. POST `/api/specs` (202 + 백그라운드 bootstrap → 파이프라인 spawn). GET `/api/specs/approval`, POST `/api/specs/approve`, POST `/api/specs/reject` (Boss 승인). 파이프라인 리뷰 API: GET/POST `/api/pipeline/reviews/:task_id/approve|reject|retry`. **Axum 0.7 라우팅 통일**: `{id}` 중괄호 패턴을 `:thread_id`/`:agent_id`/`:task_id`/`:project_id` 콜론 패턴으로 9개 라우트 일괄 치환 — 404 Not Found 완전 해결. **Pause/Resume/Thread 제어 추가**: `POST /api/pause` → `state.pipeline_running.store(false)`, `POST /api/threads/:id/approve` (stub → 실구현), `POST /api/threads/:id/reject`, `POST /api/threads/:id/retry` 신설. `POST /api/resume` 실제 구현 | 웹 UI |
| `admin.rs` | **core (핵심)** | 관리 기능 명령 및 프로젝트 관리 | CLI |
| `controller.rs` | **core (핵심)** | 워커 라이프사이클 컨트롤러, 에이전트 세션 관리 | axon-dispatcher |
| `cli.rs` | **support (지원)** | CLI 아규먼트 파싱 (clap) | main.rs |
| `dep_graph.rs` | **core (핵심)** | 의존성 그래프 생성 및 분석 | lib.rs |
| `execution_validator.rs` | **core (핵심)** | 바이너리 실행 검증 (컴파일, 실행, 타임아웃) | lib.rs |
| `rewrite_detector.rs` | **utility (유틸리티)** | 에이전트 출력물의 파괴적 덮어쓰기 감지 | lib.rs |
| `observability.rs` | **core (핵심)** | 메트릭 수집, 트레이싱, 로깅 파이프라인. `EventBusLayer` — tracing 이벤트를 EventBus로 포워딩 | 시스템 전체 |
| `events.rs` | **utility (유틸리티)** | `pub use axon_core::events::EventBus` 리익스포트 (로컬 중복 구조체 제거, 단일 EventBus 타입 통일). **server.rs에서 자동 영속화**: EventBus publish → tokio::spawn subscriber → storage.save_event() | 모든 크레이트 |
| `quarantine/` | **core (핵심)** | 격리된 레거시 데몬/서버 코드 스냅샷 (`legacy_daemon.rs`, `legacy_main.rs`, `legacy_server.rs` + JSON 스냅샷) — 참조용 보관, 컴파일 제외 | lib.rs |
| `debug_hook.rs` | **utility (유틸리티)** | 개발용 디버그 훅 | lib.rs |
| `contract/graph.rs` | **core (핵심)** | 검증용 규약(Contract) 그래프 | execution_validator |
| `contract/mod.rs` | **core (핵심)** | 규약 모듈 루트 | execution_validator |
| `bin/prompt_check.rs` | **utility (유틸리티)** | 프롬프트 검증 유틸리티 | 개발(dev) |
| `bin/drift_test.rs` | **utility (유틸리티)** | IR 드리프트 감지 유틸리티 | 개발(dev) |
| `bin/stress_test.rs` | **utility (유틸리티)** | 스트레스 테스트 유틸리티 | 개발(dev) |
| `bin/axon.rs` | **core (핵심)** | 통합 CLI (`mutate`, `replay`, `prove`, `verify`, `govern`) - Thin Governance 진입점 | CLI |
| `bin/gtk_reconnect_demo.rs` | **utility (유틸리티)** | GTK2 Unsafe vs Safe Mutation 런타임 위상 보존 검증 하네스 | 개발(dev) |
| `bin/win32_reconnect_demo.rs` | **utility (유틸리티)** | Win32 Safe Timer Evolution 검증 하네스 | 개발(dev) |
| `bin/legacy_async_demo.rs` | **utility (유틸리티)** | Legacy Async 리팩토링 검증 (Cancellation Guard) 하네스 | 개발(dev) |
| `bin/xchat_pilot.rs` | **core (핵심)** | XChat Phase 1: Read-Only Pilot (텔레메트리 Baseline 캡처 증명) | Real Repo Pilot |
| `bin/xchat_pilot_phase2.rs` | **core (핵심)** | XChat Phase 2: Tiny Mutation (고아 콜백 방지 위상 증명) | Real Repo Pilot |
| ~~`intelligence/decision.rs`~~ | **legacy (레거시)** | ~~단계 결정 엔진: 파이프라인 다음 단계를 결정~~ (Deterministic Repair Kernel로 대체) | lib.rs |
| ~~`intelligence/coordinator.rs`~~ | **legacy (레거시)** | ~~태스크 배치 코디네이터: 배치 구성, 에이전트 선택 관리~~ | lib.rs |
| ~~`intelligence/selection.rs`~~ | **legacy (레거시)** | ~~에이전트 선택 전략 및 라우팅~~ | coordinator |
| ~~`intelligence/orchestrator.rs`~~ | **legacy (레거시)** | ~~상위 수준 오케스트레이션 로직~~ | lib.rs |
| ~~`intelligence/planner.rs`~~ | **legacy (레거시)** | ~~태스크 계획 및 상세 분해~~ | lib.rs |
| ~~`intelligence/staging.rs`~~ | **legacy (레거시)** | ~~파이프라인 단계 관리~~ | lib.rs |
| ~~`intelligence/promotion.rs`~~ | **legacy (레거시)** | ~~코드 승격 (staging→seal→lock)~~ (Governance 계층으로 위임) | lib.rs |
| ~~`intelligence/commit.rs`~~ | **legacy (레거시)** | ~~코드 커밋 및 락인(lock-in) 로직~~ | lib.rs |
| ~~`intelligence/writer.rs`~~ | **legacy (레거시)** | ~~코드 생성 라이터~~ | lib.rs |
| ~~`intelligence/priority.rs`~~ | **legacy (레거시)** | ~~태스크 우선순위 스코어링~~ | coordinator |
| ~~`intelligence/rule_engine.rs`~~ | **legacy (레거시)** | ~~규칙 평가 엔진~~ | lib.rs |
| ~~`intelligence/rule_registry.rs`~~ | **legacy (레거시)** | ~~규칙 등록 및 조회~~ | rule_engine |
| ~~`intelligence/constraint_meta.rs`~~ | **legacy (레거시)** | ~~제약 조건 메타데이터~~ | lib.rs |
| ~~`intelligence/jurisprudence.rs`~~ | **legacy (레거시)** | ~~판례(Precedent) 및 사례 추적~~ | lib.rs |
| ~~`intelligence/semantic_debugger.rs`~~ | **legacy (레거시)** | ~~시맨틱 디버깅 및 진단~~ | lib.rs |
| ~~`intelligence/ir_diff.rs`~~ | **legacy (레거시)** | ~~IR Diff 계산~~ | lib.rs |
| ~~`intelligence/global_registry.rs`~~ | **legacy (레거시)** | ~~전역 상태 레지스트리~~ | lib.rs |
| ~~`intelligence/include_path_normalizer.rs`~~ | **legacy (레거시)** | ~~인클루드 경로 표준화~~ | axon-agent |
| `intelligence/language_contract/` | **core (핵심)** | 언어별 규약 (C, C++, Rust, Python, 공통) | validator |
| `intelligence/lsp/` | **core (핵심)** | LSP 연동: clangd, rust-analyzer, pyright, supervisor, session, diagnostics | lib.rs |
| `intelligence/ast/mod.rs` | **core (핵심)** | AST 소유권 검증기 추상화 (`AstOwnershipValidator`), Canonical Range 및 `RegexAstValidator` | lib.rs |
| `intelligence/ast/treesitter.rs` | **core (핵심)** | Tree-Sitter 기반 섀도우 관찰자 구현 및 Dual-Run 섀도우 로깅 | lib.rs |
| `intelligence/topology/mod.rs` | **core (핵심)** | 심볼-토폴로지 인지형 수리 스케줄러 진입점 | lib.rs |
| `intelligence/topology/symbol_graph.rs` | **core (핵심)** | 정적 심볼 의존성 추출 (`SymbolDependencyGraph`, `SymbolNode`) | scheduler.rs |
| `intelligence/topology/failure_attribution.rs` | **core (핵심)** | 컴파일 실패 시 원인 심볼 판별 (`FailureAttribution`) | scheduler.rs |
| `intelligence/topology/repair_radius.rs` | **core (핵심)** | 에러 종류별 복구 반경 결정 (`RepairRadius`) | failure_attribution.rs |
| `intelligence/topology/scheduler.rs` | **core (핵심)** | 의존성 중심성 및 오너십 기반 복구 스케줄링 (`TopologyAwareScheduler`) | lib.rs |
| `intelligence/topology/replay.rs` | **utility (유틸리티)** | 스케줄러 시뮬레이션 및 리플레이 하네스 | scheduler.rs |
| `intelligence/topology/delta_validator.rs` | **core (핵심)** | 패치 전후 토폴로지 변동 관측 (`TopologyDeltaValidator`) | mutation_sandbox.rs |
| `intelligence/patch_ir.rs` | **core (핵심)** | AST 패치 연산 정의 IR 계층 | mutation_sandbox.rs |
| `intelligence/signature_drift.rs` | **core (핵심)** | 함수 시그니처 변동 검출 | patch_ir.rs |
| `intelligence/mutation_sandbox.rs` | **core (핵심)** | 파일 쓰기 전 Dry-run 패치 및 안전성 샌드박스 | patch_ir.rs |
| `intelligence/provenance.rs` | **core (핵심)** | 패치 생성 원인 추적용 불변 출처 기록 (`PatchProvenance`) | patch_ir.rs |
| `intelligence/observatory.rs` | **core (핵심)** | AST 섀도우 런 관측 기록소 및 통계 생성 (P5-5.5) | mutation_sandbox.rs |
| `intelligence/stability_matrix.rs` | **core (핵심)** | 코퍼스 기반 AST Mutation 라운드트립 안정성 실측 하네스 (P5-6b) | observatory.rs |
| `intelligence/heatmap.rs` | **core (핵심)** | 위험 지형도 스코어링 및 Authoritative 승격 게이트 (P5-6d) | stability_matrix.rs |
| `intelligence/causality.rs` | **core (핵심)** | 시스템 전체 인과율 추적 (`StateTransitionRecord`) (P5-7a) | heatmap.rs |
| `intelligence/determinism_harness.rs` | **core (핵심)** | 스케줄러/토폴로지 결정론성 리플레이 검증 (`verify_determinism`) (P5-7b) | causality.rs |
| `intelligence/rollback.rs` | **core (핵심)** | 토폴로지 파괴 시 소유권 스냅샷까지 복구하는 롤백 엔진 (P5-7c) | causality.rs |
| `governance/atomic_io.rs` | **core (핵심)** | 원자적 파일 쓰기(`write_json_atomic`)로 충돌 방지 (P6-1) | store.rs |
| `governance/store.rs` | **core (핵심)** | 단일 IO 게이트웨이 권위 상태 관리 (`GovernanceStore`) (P6-1) | atomic_io.rs |
| `governance/circuit_breaker.rs` | **core (핵심)** | 무한 수리 루프 차단용 `FailureBudget` 및 서킷 브레이커 (P6-2) | causality.rs |
| `governance/lease.rs` | **core (핵심)** | 에이전트 좀비화 방지용 TTL/소유권 회수(`OwnershipLease`) (P6-3) | store.rs |
| `governance/simulation.rs` | **core (핵심)** | 운영체제급 장애 주입 시뮬레이션 하네스 (P6-SIM) | lease.rs |
| `governance/determinism.rs` | **core (핵심)** | 시스템 상태 해시 기반 Cross-layer Race 검증기 (P6-SIM) | simulation.rs |
| `intelligence/mutation_intent.rs` | **core (핵심)** | 시맨틱 변이 의도 정의 및 최소성 강제 (P5-6e) | shadow_mutator.rs |
| `intelligence/shadow_mutator.rs` | **core (핵심)** | 파싱/출력 실측 및 Semantic Equivalence 섀도우 검증 (P5-6e) | intelligence/mod.rs |
| `intelligence/semantic_tokens.rs` | **core (핵심)** | 파서 중립적 토큰 정규화 계층 (P5-7a) | canonicalizer.rs |
| `intelligence/canonicalizer.rs` | **core (핵심)** | 정책 적용을 통한 `CanonicalSemanticForm` 변환 (P5-7b) | semantic_authority_gate.rs |
| `intelligence/semantic_distance.rs` | **core (핵심)** | 양방향 토폴로지 인지형 시맨틱 거리 측정기 (P5-7c) | semantic_authority_gate.rs |
| `intelligence/semantic_authority_gate.rs` | **core (핵심)** | 최종 승인 게이트. `CanonicalSemanticForm` 검증 (P5-7d) | intelligence/mod.rs |
| `intelligence/edit_plan.rs` | **core (핵심)** | 최소 바이트 편집(`ByteEdit`) 변환 규약 `StableEditPlan` (P5-8a) | surgical_editor.rs |
| `intelligence/tree_sitter_locator.rs` | **core (핵심)** | Tree-sitter를 '고정밀 좌표 추출기'로만 격하시켜 활용 (P5-8b) | edit_plan.rs |
| `intelligence/surgical_editor.rs` | **core (핵심)** | 포매팅 엔트로피 보존형 바이트 정밀 수술(Surgery) (P5-8c) | intelligence/mod.rs |
| `intelligence/anchor_validator.rs` | **core (핵심)** | TOCTOU 부패 방지용 앵커(`SemanticAnchor`) 실시간 재검증 (P5-8d) | surgical_editor.rs |
| `intelligence/surgical_replay.rs` | **core (핵심)** | 파이프라인 무결성을 실측하는 리플레이 하네스 (P5-8e) | shadow_mutator.rs |
| `intelligence/intent_lowering/` | **core (핵심)** | SAFE_SUBSET_V1 하향 변환 및 `PromotionReport` 모듈 (P5-8f) | surgical_editor.rs |
| `intelligence/replay/` | **core (핵심)** | 통계적 거버넌스 및 `PromotionEngine`. Phase E~G (Runtime Event Topology, GTK2 GObject Hell, Trace Layering, Immunology Genealogy) 붕괴 관측 및 면역 체계 모듈 포함: `c_topology_strike.rs`, `catastrophe_pressure.rs`, `closed_loop_harness.rs`, `corpus_fingerprint.rs`, `corpus_runner.rs`, `gtk2_gobject_hell.rs`, `gtk_baseline_seal.rs`, `gtk_stage1_2_strike.rs`, `gtk_stage3_collapse.rs`, `gtk_topology_strike.rs`, `harness.rs`, `immunology_genealogy.rs`, `lineage_taxonomy.rs`, `metrics_aggregator.rs`, `mod.rs`, `orchestrator.rs`, `parser_freeze.rs`, `policy_audit.rs`, `promotion_engine.rs`, `regression_snapshot.rs`, `runtime_event_topology.rs`, `strike_test.rs`, `trace_layering.rs`, `win32_topology_strike.rs` | intelligence/mod.rs |
| `intelligence/corpus/` | **core (핵심)** | 현실 세계 레거시 엔트로피 수집, 클러스터링, 캠페인 및 Corpus Governance 파이프라인 (P5-8h): `campaign_manifest.rs`, `campaign_runner.rs`, `catastrophe_archive.rs`, `corpus_executor.rs`, `corpus_fingerprint.rs`, `corpus_governance.rs`, `corpus_ingestor.rs`, `corpus_seal.rs`, `divergence_cluster.rs`, `entropy_profiler.rs`, `entropy_snapshot_store.rs`, `failure_classifier.rs`, `failure_lineage.rs`, `hierarchical_topology.rs`, `mod.rs`, `mutation_campaign.rs`, `physical_mount.rs`, `replay_seed.rs`, `repo_fetcher.rs`, `rox_filer_hotspot.rs`, `runtime_adjacency.rs`, `workspace_materializer.rs`, `xchat_hotspot.rs` | intelligence/mod.rs |
| `tests/promotion/` | **core (핵심)** | Mock Promotion Validation Suite. 의도적 실패 및 유니코드 변이 테스트 (P5-8g.1) | replay/promotion_engine.rs |
| `tests/failure_cascade/` | **core (핵심)** | P6-SIM-T1: 크래시 주입 통합 테스트 스위트 | governance/ |
| `intelligence/mutation/` | **core (핵심)** | 위상 변이 트랜잭션 제어. 의도(Intent) 정의, 변이 경계 락(Boundary Lock) 및 안전 봉투(Envelope) 제어 | intelligence/mod.rs |
| `intelligence/evolution/` | **core (핵심)** | 통제된 소프트웨어 진화 워크플로우(Replay 기반 승인 루프) 및 런타임 위상 변형 시각화(`drift_visualizer`) | intelligence/mod.rs |
| `intelligence/telemetry/` | **core (핵심)** | 물리적 런타임 센서, 원시 텔레메트리 캡처 및 Jitter를 제거하는 런타임 병리 압축기(`causality_compressor`) | intelligence/mod.rs |

### /crates/axon-agent/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | AgentRuntime: process_task, generate_ir, generate_ir_with_context, process_spec_analysis, repair_ir_pass, HotRuleCache. `generate_ir_with_context`: `floor_char_boundary(ceiling)` 동적 천장(`min(spec.len(), 48000)`) UTF-8 안전 자르기. `process_spec_analysis`: forbidden 시스템 라이브러리 컴포넌트 Phase 4-3 필터 (`Path::file_stem()` exact match + `eq_ignore_ascii_case`, starts_with 오탐 수정). **Protocol Downsizing**: `_effective_rework = false` 강제 (SEARCH/REPLACE 영구 폐기). 출력 포맷 단일화 — markdown code block만 요구. 파서 fallback 체인: (1) `extract_axon_patch_v2` (2) `extract_code_block()` 신규 함수 (3) `extract_cpp_c_code()` C/C++ raw 패턴 추출 (4) raw text fallback. `apply_hunks()` 함수 + `extract_json()` fallback 완전 제거. `final_code = full_code` 단순화. `review_proposal()` Senior 프롬프트 ULTRA LIGHT — `FIX_HINT:` 프리픽스 제거, `First line: [APPROVE] or [REJECT]. Next lines: free text feedback.` **PARSER FAIL 시 샌드박스 보존**: `remove_file()` → `rename()` `.failed` 확장자로 스왑, retry 시 original_code 유지 | axon-daemon, axon-model |
| `persona.rs` | **core (핵심)** | 에이전트 페르소나 정의 및 주입 | axon-daemon |
| `composer.rs` | **support (지원)** | 코드 합성 유틸리티 | lib.rs |
| `lounge.rs` | **support (지원)** | 노가리(Lounge) 채널 상호작용 지원 | axon-daemon |

### /crates/axon-dispatcher/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | Dispatcher: enqueue_task, pop_ready_task, 라운드 로빈 스케줄링, 큐 크기 제한 | axon-daemon |

### /crates/axon-storage/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | Storage: SQLite + WAL, 비동기 배치 쓰기, 스키마 마이그레이션, 파일 수준 락킹, Dead Letter Queue. **FIFO flush drain**: flush signal 수신 시 `rx.try_recv()`로 모든 선행 쓰기 메시지를 선행 소진하여 WAL flush의 완전한 sync barrier 보장 | axon-daemon |

### /crates/axon-model/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **critical (치명)** | ModelDriver 트레이트 및 GeminiDriver, ClaudeDriver, OpenAIDriver, OllamaDriver 구현 | axon-agent |

### /crates/axon-platform-win32/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **core (핵심)** | Win32Contract 구조체 및 임베디드 규약 파일 (subsystem, winmain, message loop, wndproc, rendering) | axon-ir-validator |

### /crates/axon-ir-validator/src/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `lib.rs` | **core (핵심)** | PlatformValidator: validate_source_code (20개 이상의 Win32 규칙), validate_binary_subsystem (PE 헤더 파싱), validate_spec | axon-daemon |

### /studio/

| 파일 | 역할 | 책임 | 관련 시스템 |
|------|------|---------------|-----------------|
| `src/App.tsx` | **core (핵심)** | Boss Board UI 루트. 좌측 6채널 네비게이션(Dashboard/Work/Office/Boss/Nogari/Signals). WebSocket(`/ws`) 이벤트 구독, `/api/*` REST 폴링. **Dashboard 2열**: `totalSignals`(시그널) + `nogariCount`(노가리) 병렬 표시. `fetchStatus()`에서 `data.nogari_count` 읽음. **ThreadDetail 연동**: `handleApprove` → `POST /api/threads/{id}/approve` 호출, `onRefresh={fetchThreads}` props 전달 | server.rs |
| `src/api/socket.ts` | **core (핵심)** | WebSocket 클라이언트 (`ws://host/ws`). EventBus 이벤트를 JSON으로 수신 → 콜백 리스너에 디스패치 | App.tsx |
| `src/components/BossBoard.tsx` | **core (핵심)** | Semantic Governance Console. `GET /api/specs/approval` 폴링 + Approve/Reject 버튼. `POST /api/specs`, `GET /api/specs/status/{id}`. Semantic risk 관리 (risks/decide). **Pipeline Reviews 섹션**: `GET /api/pipeline/reviews` 2초 폴링, 3회 실패 태스크 목록 + Approve/Reject/Retry 버튼, 좌측 사이드바 카운트 배지 | server.rs |
| `src/components/ThreadDetail.tsx` | **core (핵심)** | 스레드 상세 보기 (posts 로드, 포맷 렌더링). **2초 polling**: `useEffect` 내 `setInterval(fetchPosts, 2000)` + `clearInterval` cleanup (WebSocket `PostCreated` 이벤트 기반 개선 전까지). **3종 제어 버튼**: Approve(`POST /api/threads/{id}/approve`) + Reject(`POST /api/threads/{id}/reject`) + Retry(`POST /api/threads/{id}/retry` + feedback input). `onRefresh` prop으로 부모 데이터 갱신 | server.rs |
| `src/components/Office.tsx` | **core (핵심)** | 에이전트 관리 화면. `GET /api/agents`, `POST /api/agents/hire`, `POST /api/agents/{id}/fire` | server.rs |
| `src/components/Lounge.tsx` | **support (지원)** | 노가리 채널 (실시간 메시지 로그) | server.rs |

## 데이터 흐름 (Data Flow) - 시맨틱 거버넌스 파이프라인

```
[외부 코퍼스 / 레거시 프로젝트]
    ↓
[axon-daemon::corpus_ingestor] → Reproducible Corpus Snapshot 동결 생성
    ↓
[BootstrapManager::run_v4] 결정론적 섀도우 검증 루프:
    │
    ├─ Stage::CorpusHarvesting (엔트로피 수집)
    │   ├─ EntropyProfiler: 대상 파일의 위험 지형도(RiskClass) 계량화
    │   ├─ MutationCampaign: SAFE_SUBSET_V1 대량 주입 및 리플레이 통계 확보
    │   └─ DivergenceCluster: 실패 패턴을 Root Cause 단위로 군집화
    │
    ├─ Stage::SemanticAnalysis (의미론적 해석)
    │   ├─ TreeSitter Locator: 고정밀 앵커 및 좌표 추출 (수술 위치 확보)
    │   ├─ Canonicalizer: 파서/포매터 중립적인 CanonicalSemanticForm으로 정규화
    │   └─ SemanticPolicyAuditor: "harmless + harmless = catastrophic" 정책 충돌 감사
    │
    ├─ Stage::ShadowExecution (섀도우 수술 및 검증)
    │   ├─ SurgicalEditor: 포매터 개입 없이 들여쓰기를 보존하는 바이트 정밀 편집
    │   ├─ AnchorValidator: TOCTOU 앵커 표류(Drift) 방지 및 재검증
    │   └─ DeterminismHarness: 10,000회 리플레이를 통한 토폴로지 보존 및 결정론 실측
    │
    └─ Stage::AuthoritativePromotion (최종 권위 승격)
        ├─ PromotionEngine: 통계적 거버넌스 Threshold(안정성 99.9% 등) 검토
        └─ GovernanceStore: 원자적 파일 쓰기(Atomic IO) 및 SYSTEM_STATE_HASH 동결 락인
```

## 아키텍처 의존성 그래프

```
axon-core (모델 및 토폴로지 추상화) ←─ axon-model (LLM 드라이버 추상화)
                       ↓
               axon-ir (IR 및 Canonical Form) ←─ axon-platform-win32 (플랫폼 규약)
                       ↓                      ↓
               axon-agent (섀도우/의도 생성)     axon-ir-validator (물리/구문 검증기)
                       ↓                      ↓
               axon-daemon (Deterministic Repair Kernel & 관제탑)
                       ↓
               axon-storage (SQLite / WAL 영속성 저장소)
```

## 중요도 범례

| 수준 | 의미 | 예시 |
|-------|---------|---------|
| **critical (치명)** | 손상 시 시스템 붕괴 및 무결성 상실 | `surgical_editor.rs`, `promotion_engine.rs`, `schema/types.rs` |
| **core (핵심)** | 거버넌스 파이프라인 및 코퍼스 수집 기능 | `intelligence/corpus/*`, `intelligence/replay/*` |
| **support (지원)** | 부가적 로깅, 섀도우 런 보조 | `observability.rs`, `debug_hook.rs` |
| **utility (유틸리티)** | 개발 및 단일 테스트 도구 | `bin/stress_test.rs` |
| **legacy (레거시)** | 과거 LLM 자유 코드 생성 방식 (폐기됨) | `intelligence/decision.rs`, `intelligence/writer.rs` |

---

## 런타임 경계 및 리스크 요인

| 경계선 | 적용 기술 | 제어 모델 | 주요 위험 요인 (리스크) |
|----------|-----------|-------------|------|
| **Daemon Process** | tokio 멀티 스레드 | 비동기 상태 머신 | 토폴로지 폭발(Topology Explosion), 데드락 |
| **Storage** | SQLite + WAL | 배치 MPSC 쓰기 | 쓰기 경합, WAL 파일 비대화 |
| **Web UI (Boss Board)** | axum + tower-http | 실시간 SSE 통신 | 연결 유실, 관제탑(인간) 응답 지연 |
| **External LLM** | reqwest HTTPS | 섀도우 내 격리 실행 | 환각(Hallucination), 비정형 출력 엔트로피 |
| **File System** | `atomic_io.rs` | `GovernanceStore` 통제 | TOCTOU 부패, 불완전한 쓰기(Partial write) |

---

## 단일 진실 공급원 (Source of Truth)

현재 AXON은 AST나 소스코드가 아닌 **"정규화된 시맨틱 폼"**과 **"거버넌스 통계"**를 유일한 진실로 삼습니다.

| 시스템 | 결정 권한 | 보관 위치 | 검증 주체 |
|--------|-----------|----------|------------|
| **CanonicalSemanticForm** | 시맨틱 일치 여부 판별 | `ProcessMemory` | `SemanticAuthorityGate` |
| **Promotion Metrics** | 섀도우 통계 및 승격 자격 | `axon-storage (SQLite)` | `PromotionEngine` |
| **Corpus Fingerprint** | 치명적 붕괴 패턴(지문) 레지스트리 | `데이터베이스 / 레지스트리 맵` | `DivergenceCluster` |
| **SYSTEM_STATE_HASH** | 파이프라인 Race Condition 감시 | `ProcessMemory` | `DeterminismHarness` |
| **ImmutableConstraints** | 초기 사양 불변 제약 | `{sandbox}/immutable_constraints.json` | `axon-agent` |
| **Platform Contracts** | 시스템 규약 (Win32 등) | `임베디드 내장 (include_str!)` | `axon-ir-validator` |

---

## 데이터 수정 소유권 (Mutation Ownership)

AXON의 권한 분리 철학에 따라, 파일 시스템 쓰기는 오직 **거버넌스 게이트**를 통해서만 가능합니다.

### 오직 axon-daemon (Governance)만 수행 가능:
- **원자적 파일 쓰기**: `GovernanceStore`를 통한 바이트 수정 적용 및 락인.
- **최종 승격(Promotion) 판정**: `PromotionEngine`을 거친 통계적 수락 및 반려.
- **롤백 및 복구**: 토폴로지 파괴 감지 시 과거 소유권/상태로의 복귀(`rollback.rs`).
- **코퍼스 격리(Quarantine)**: 진본성이 떨어진 코퍼스 배제 (`corpus_governance.rs`).

### 오직 axon-agent (Shadow)만 수행 가능:
- **시맨틱 의도(Intent) 생성**: 섀도우 환경에서 코드를 어떻게 바꿀지 `MutationIntent` 제안.
- **LLM 호출 및 추론**: 코드 템플릿(SAFE_SUBSET) 작성 및 리뷰 의견 텍스트화.

### 오직 axon-ir (Interpreter)만 수행 가능:
- **Canonical 정규화**: 파서(Tree-sitter)의 출력을 `CanonicalSemanticForm`으로 변환.
- **위험 지형도 분석**: Entropy Profiler를 통한 파일 분석.

---

## 승격 라이프사이클 (Promotion Lifecycle)

코드 수정은 더 이상 "작성 → 커밋"이 아니라 **"섀도우 제안 → 실측 검증 → 승인"**의 거버넌스를 따릅니다.

```
[Shadow Proposal] (의도 제안)
  ↓ LLM 혹은 템플릿이 SAFE_SUBSET_V1 형태의 변이를 제안
[Mutation Campaign] (대규모 캠페인 실측)
  ↓ 10,000회 리플레이, 토폴로지 보존율, 앵커 생존율 측정
[Audit & Triage] (감사 및 분류)
  ↓ DivergenceCluster가 실패 패턴 분석, SemanticPolicyAuditor가 충돌 감시
[Governance Approval] (거버넌스 승인)
  ↓ PromotionEngine이 Threshold 만족 여부를 판단하여 보스(관제탑)에 리포트
[Authority Lock] (권위 락인)
  └─ GovernanceStore를 통해 파일 시스템에 원자적(Atomic) 적용 및 SYSTEM_STATE_HASH 동결
```

---

## 보호 시스템 ("임의 수정 금지" 성역 영역)

다음 파일들을 변경하는 것은 AXON의 **헌법(거버넌스 체계)** 자체를 수정하는 것으로, 철저한 통제가 필요합니다.

| 파일 경로 | 잠금 사유 | 위반 시 카타스트로피 (리스크) |
|------|--------|--------------------------------|
| `intelligence/surgical_editor.rs` | 바이트 정밀 수술기. 포매터를 호출하지 않음 | 포매팅 엔트로피 대폭발, 파이프라인 무한 루프 |
| `intelligence/replay/promotion_engine.rs` | 승격 거버넌스 Threshold 판단 로직 | 불안정한 코드가 Master로 유입되어 토폴로지 붕괴 |
| `governance/store.rs` | 유일한 파일 I/O 게이트웨이 | Race condition, TOCTOU 부패, 시스템 영구 손상 |
| `intelligence/corpus/corpus_governance.rs`| 코퍼스 진본성 판단 기준 | 왜곡된 AI 생성물이 섞여 통계적 실측치 오염 |

### 자유 수정 가능 영역 (Safe Modification Zones)

다음 영역은 아키텍처 차원의 전면 검토 없이도 수정이 가능합니다:

- `intelligence/corpus/entropy_profiler.rs` — 새로운 휴리스틱/메트릭 지표 추가
- `intelligence/lsp/*` — 언어 서버 연동 확장
- `server.rs` — Boss Board (UI) 관제 기능 개편 및 시각화 추가
- `axon-ir/src/parser/*` — 신규 포맷 및 파서 룰 추가

---

## 📛 파일명 치환 규칙 (Filename Normalization Rules)

본 규칙은 GEMINI.md 및 앞으로 생성되는 모든 마일스톤 버전 문서에 적용된다.

### 1. 한글 파일명 → 영문 표준 파일명 치환 (Substitution Table)
| 원문 (한글) | 표준 파일명 | 비고 |
|---|---|---|
| 노가리.md | Nogari.md | 에이전트 라운지 잡담 로그 |
| 아키텍쳐.md | ARCHITECTURE_AXON.md | 전체 프로젝트 설계 바이블 |

### 2. 적용 범위
- 마일스톤 문서(mile_stone/v*.md) 작성 시 위 파일명을 반드시 표준 파일명으로 기재.
- 코드(Rust), 프롬프트 템플릿, 노가리 채널 로그, 릴리즈 노트 등 모든 산출물에서 동일하게 적용.
- 기존 문서에서 한글 파일명이 발견되면 수정 없이 두되, 신규 생성 시에는 표준 파일명을 사용.

### 3. 예시 (Before → After)
- "노가리.md에 소회를 남겨라" → "Nogari.md에 소회를 남겨라"
- "아키텍쳐.md를 갱신하라" → "ARCHITECTURE_AXON.md를 갱신하라"
