# AXON v0.0.26 — Bootstrap Controller

## 🎯 목적

이 문서는 AXON v0.0.26의 4개 핵심 문서를 순차적으로 실행하고 검증하는 **통합 제어 스펙**이다.

각 단계는 다음 사이클을 따른다:

```
READ → GOAL 확인 → PLAN → IMPLEMENT → VALIDATE → NEXT
```

---

# 📚 대상 문서

1. Header Include Inference + Generator
2. CPP Include + Link Dependency Generator
3. CMake Auto Generator
4. Runtime Debug Loop

---

# 🧠 LLM Coding Principles

## 🚀 Core Rules

1. **Think Before Coding**

   * 요구사항이 모호하면 질문한다
   * 최소 1개 이상의 접근 방식 제시
   * 가장 단순한 해법부터 선택

2. **Simplicity First**

   * 최소 코드
   * 불필요한 추상화 금지
   * 가독성 유지

3. **Minimal Changes**

   * 전체 재작성 금지
   * 필요한 부분만 수정
   * diff 기반 변경

4. **Goal-Oriented Execution**

   * Goal → Plan → Implementation → Validation
   * 각 단계는 검증 가능해야 함

5. **No Hallucinated APIs**

   * 존재하지 않는 API 금지
   * 불확실 시 질문

6. **Stable Code Protection**

   * 검증된 코드 수정 금지
   * 영향 범위 최소화

7. **Context Confirmation**

   * 항상 맥락 확인
   * 부족하면 요청

---

# 🧩 GLOBAL EXECUTION RULES

## Rule 1 — Stage Isolation

각 문서는 독립적으로 실행된다.
이전 단계 결과만 사용한다.

## Rule 2 — Retry Scope Enforcement

실패 시 전체 재실행 금지
→ 반드시 해당 단계만 재시도

## Rule 3 — Minimal Context

LLM에 전체 프로젝트를 주지 않는다
→ 필요한 파일만 제공

---

# 🏗️ STAGE 1 — HEADER INCLUDE GENERATOR

## 🎯 Goal

헤더에서 최소 include + forward declaration 생성

## 📥 Input

* Header (.h)

## 📤 Output

* include block
* forward declarations

## ⚙️ Plan

1. 타입 추출
2. 타입 분류 (builtin / std / project)
3. include 결정
4. forward declaration 결정

## ✅ Validation

* 컴파일 가능해야 함
* 불필요 include 없음
* missing symbol 없음

## ❌ Failure → Retry

Scope: HeaderOnly

---

# 🏗️ STAGE 2 — CPP DEPENDENCY GENERATOR

## 🎯 Goal

.cpp include + link dependency 자동 생성

## 📥 Input

* .cpp
* 대응 .h
* registry

## 📤 Output

* include list
* link dependencies

## ⚙️ Plan

1. symbol usage 추출
2. registry 매핑
3. include 생성
4. object dependency 생성

## ✅ Validation

* 컴파일 성공
* undefined reference 없음

## ❌ Failure → Retry

Scope: ImplementationOnly

---

# 🏗️ STAGE 3 — CMAKE GENERATOR

## 🎯 Goal

dependency graph → CMakeLists.txt 생성

## 📥 Input

* dependency graph
* file registry

## 📤 Output

* CMakeLists.txt

## ⚙️ Plan

1. target 분류 (lib / exe)
2. dependency → link 변환
3. CMake 렌더링

## ✅ Validation

* cmake configure 성공
* build 성공

## ❌ Failure → Retry

Scope: Full

---

# 🏗️ STAGE 4 — RUNTIME DEBUG LOOP

## 🎯 Goal

실행 결과 기반 자동 수정 루프

## 📥 Input

* 실행 결과 (stdout / stderr / exit code)

## 📤 Output

* RetryPlan
* Hint

## ⚙️ Plan

1. 실행
2. 에러 캡처
3. 원인 분류
4. hint 생성
5. retry scope 결정

## ✅ Validation

* 프로그램 정상 종료
* expected output 만족

## ❌ Failure → Retry

Scope: ImplementationOnly → 필요 시 Full escalation

---

# 🔁 GLOBAL LOOP

```
Stage1 → Stage2 → Stage3 → Stage4
            ↑                ↓
            └──── Retry ─────┘
```

---

# 🧪 TEST STRATEGY

## Unit Tests

* Header parsing 테스트
* Symbol extraction 테스트

## Integration Tests

* .h → .cpp → build

## Runtime Tests

* 실행 결과 검증

---

# 🚨 FAILURE HANDLING

## 1. 반복 실패 (N회 이상)

→ Scope 확장

## 2. 구조 오류

→ Skeleton 단계로 롤백

## 3. 알 수 없는 오류

→ Full 재생성

---

# 📌 핵심 원칙 요약

* LLM은 “생성기”가 아니라 “부분 해결기”
* 시스템은 “의존성 해결자”
* Validator는 “가이드”

---

# 🏁 종료 조건

* Build 성공
* Runtime 성공
* Retry 없음

---

# 🧭 한 줄 결론

> 이 문서는 LLM을 통제하는 것이 아니라
> **LLM이 실패해도 무너지지 않는 시스템을 만드는 설계다**

