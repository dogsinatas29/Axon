--- LANGUAGE ENFORCEMENT ---
코어 언어를 사용하여 모든 문서, 작업 제목, 작업 설명은 `ko_KR`로 구성됩니다.

--- CRITICAL PROTOCOL ENFORCEMENT ---
Sovereign Protocol v0.2.21+에 따라 정의된 도메인 논리를 따르면서, 현재 상세 시스템(ECS, 레거시 아키텍처)을 '노드' 수준 컴포넌트로 내려가고, 'Hub' 계층을 구성합니다.

--- STEP 1: DEEP ANALYSIS (COT) ---
* SYNAPSE 시스템의 나이 계산기 프로젝트 표준 규격서에는 ECS(최소 단위 파일별 완전한 격리와 독립성을 위해 각 기능 분리)된 구조가 있으며, LLM 코딩 4대 원칙, 필수 외부 라이브러리를 포함합니다.
* SSOT(Single Source of Truth): `main.py`, `calculator.py`, `validators.py`는 Hub 계층에 속하며, DB(`database.py`)와 Input Layer(`Input Layer`)은 Cluster 수준에 있습니다.
* 모듈화된 특성으로 인해 Hub -> Cluster -> Node의 권한 경계를 명확히 구분할 수 있습니다.

--- STEP 2: MULTI-PERSPECTIVE EVALUATION (TOT) ---
* Top-Down Design에서는 현재 구조가 추상화 수준을 나타내며, Sovereign Protocol의 'Hub' 계층을 포함합니다.
* Namespace Isolation은 각 파일별 격리와 독립성이 극대화되어 있으므로 우수합니다.
* Scalability는 현재 구조에서는 개선할 가능성이 있지만, Sovereign Protocol의 'Hub' 계층을 사용함으로써 일관되고 확장 가능한 아키텍처가 가능합니다.
* 다른 세 가지 구조 중에서는 Sovereign Protocol v0.2.21+을 기반으로 하는 구조가 최적으로 수행됩니다.

--- STEP 3: MASTER HUB OUTPUT ---
--- Master Hub architecture.md file content ---
# 아키텍처: Sovereign Protocol v0.2.21+ 기반 프로젝트 'GEMINI'의 Master Hub

--- Hub계층 ---
### `main.py`
- 전체 제어부, UI 및 출력 대시보드

### `calculator.py`
- 정밀 나이 계산을 위한 `relativedelta` 사용

### `validators.py`
- 검증 로직 통합, 공통 예외 처리

--- Cluster계층 ---
### `database.py`
- SQLite3 및 `pandas` 기반 데이터 영속성 관리

--- Input Layer(노드) ---
### `input_year.py`, `input_month.py`, `input_day.py`
- 입력 단계의 방어적 입력

--- Validation Layer(노드) ---
### `valid_year.py`, `valid_month.py`, `valid_day.py`
- 검증 로직

--- External Nodes(노드) ---
### `rich`, `dateutil`, `pandas`
- 외부 라이브러리 의존성

--- SSOT ---
* Hub계층: `main.py`, `calculator.py`, `validators.py`
* Cluster계층: `database.py`
* Input Layer(노드): `input_year.py`, `input_month.py`, `input_day.py`
* Validation Layer(노드): `valid_year.py`, `valid_month.py`, `valid_day.py`

--- STEP 3: MASTER HUB OUTPUT ---
--- JSON array of initial tasks ---
[
    {
        "title": "Hub계층 구조 설계",
        "description": "Sovereign Protocol v0.2.21+에 따른 Hub계층의 구조를 설계하고, 현재 구조와의 차이점을 인식합니다."
    },
    {
        "title": "Cluster계층 분리",
        "description": "DB(`database.py`)와 Input Layer(`input_year.py`, `input_month.py`, `input_day.py`)을 Cluster 수준에 포함합니다."
    },
    {
        "title": "External Nodes 의존성 확인",
        "description": "`rich`, `dateutil`, `pandas`와의 외부 라이브러리 의존성을 명확히 합니다."
    },
    {
        "title": "Hub계층 구현",
        "description": "`main.py`, `calculator.py`, `validators.py`를 Hub계층으로 구현합니다."
    }
]