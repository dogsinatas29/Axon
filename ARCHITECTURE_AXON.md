# ARCHITECTURE_AXON: AXON Test Project

## [SNR] 👴 Senior: Initializing from captured context.

# SYNAPSE Architecture & Discovery Rules (설계 및 발견 규칙)

This document defines the rules for how SYNAPSE discovers, parses, and visualizes the project architecture.
본 문서는 SYNAPSE가 프로젝트 아키텍처를 발견, 파싱 및 시각화하는 규칙을 정의합니다.

---

## 1. Node Inclusion Rules (노드 포함 규칙)
- **Real Path Priority (실제 경로 우선)**: Only files and folders that actually exist in the project root (e.g., `src/`, `prompts/`) are valid nodes.
- **Icon Standards (아이콘 표준)**: 
    - Folder nodes MUST be prefixed with the 📁 icon.
    - File nodes MUST be prefixed with the 📄 icon.
- **Core Components (중추 컴포넌트)**: Critical system logic must always be placed in the top-level cluster.

## 2. Exclusion & Refinement Rules (제외 및 정제 규칙)
- **Code Block Isolation (코드 블록 격리)**: Text inside multi-line code blocks is excluded from scanning.
- **Inline Code Protection (인라인 코드 보호)**: Filenames wrapped in single backticks (`...`) do not trigger node creation.
- **Comment Ignores (주석 무시)**: Text inside HTML comments `<!-- ... -->` is ignored.
- **Node Diet (최적화)**: Non-architectural documents and build artifacts are excluded:
    - `README.md`, `README_KR.md`, `CHANGELOG.md`, `.vsix`, `.js.map`
    - `node_modules`, `.git`, `dist`, `build`, `ui`

## 3. Edge & Flow Definitions (엣지 및 흐름 정의)
- **Execution Flow Priority (실행 흐름 우선)**: Connections (`-->`) should represent actual **'Execution Flow'**.
- **Layer Compliance (레이어 준수)**: Connections should follow: `Discovery` -> `Reasoning` -> `Action`.


## Current Status
- [ ] System Setup
