# 🚀 프로젝트: AXON 완전체 C 나이 계산기 (v0.5-Sovereign)

> **[!IMPORTANT]**
> 본 문서는 AXON v0.0.30 시스템의 '완결성' 검증을 위한 최종 규격서입니다.
> SQLite3 C API를 직접 제어하며, 모든 공정은 엄격한 의미론적 봉인(Sealing)을 따릅니다.

---

## 1. 기술 스택 및 의존성 (Technical Stack)

| 라이브러리 | 역할 | 버전/표준 |
| :--- | :--- | :--- |
| **`sqlite3`** | 로우 레벨 DB 엔진 제어 | C API (v3.x) |
| **`time.h`** | 정밀 날짜 연산 및 시각 추출 | ISO C99 |
| **`stdio.h/stdlib.h`** | 입출력 및 메모리 관리 | ISO C99 |

---

## 2. 데이터베이스 사양 (SQLite3 Implementation)

### 2-1. 스키마 (Schema)
```sql
CREATE TABLE user_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    birth_year INTEGER NOT NULL,
    birth_month INTEGER NOT NULL,
    birth_day INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 2-2. SQLite3 C API 필수 사용 규약
AI 에이전트는 다음의 물리적 단계를 반드시 준수하여 `database.c`를 구현해야 한다.
1. **Connection**: `sqlite3_open()`을 통한 세션 개시.
2. **Statement**: 모든 쿼리는 `sqlite3_prepare_v2()`로 컴파일.
3. **Binding**: `sqlite3_bind_int()`, `sqlite3_bind_text()`를 사용하여 SQL Injection 원천 차단.
4. **Execution**: `sqlite3_step()`을 통한 실행 및 `SQLITE_DONE`/`SQLITE_ROW` 상태 체크.
5. **Finalization**: `sqlite3_finalize()` 및 `sqlite3_close()`를 통한 자원 해제 필수.

---

## 3. 컴포넌트 및 함수 원형 (Architectural IR Contract)

### 3-1. `database.c/h` (Persistence Layer)
- `int db_init(const char *db_name)`: DB 연결 및 테이블 생성 (실패 시 -1 반환).
- `int db_add_user(const char *name, int y, int m, int d)`: 사용자 추가.
- `int db_list_users()`: 저장된 모든 사용자 목록 출력.
- `void db_close()`: 연결 종료.

### 3-2. `calculator.c/h` (Logic Layer)
- `int calc_man_age(int b_y, int b_m, int b_d)`: 만나이 계산.
- `int calc_korean_age(int b_y)`: 한국 나이 계산.
- `struct tm get_current_time()`: 시스템 현재 시각 획득.

### 3-3. `validators.c/h` (Validation Layer)
- `int is_valid_date(int y, int m, int d)`: 날짜 유효성 및 윤년 검증.
- `int is_leap_year(int y)`: 윤년 여부 판단.

### 3-4. `main.c` (Control Layer)
- 사용자로부터 이름, 생년월일을 입력받아 `validators`로 검증 후 `database`에 저장하고 `calculator`로 계산 결과를 출력하는 메인 루프 구현.

---

## 4. 완결성 검증 기준 (Success Criteria)

1. **컴파일 성공**: `gcc main.c database.c calculator.c validators.c -lsqlite3 -o axon_age` 명령이 경고 없이 통과되어야 함.
2. **데이터 영속성**: 프로그램을 종료했다가 다시 켰을 때, 이전에 입력한 이름과 나이가 SQLite DB에서 정상적으로 조회되어야 함.
3. **물리적 무결성**: 생성된 소스 코드에 주석Placeholder(`// TODO`)나 AI의 변명이 포함되어서는 안 됨.
4. **의미론적 봉인**: 모든 함수는 명세서에 정의된 원형과 기능을 100% 일치시켜야 함.