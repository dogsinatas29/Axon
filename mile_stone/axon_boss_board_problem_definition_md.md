# AXON Boss Board Problem Definition System

> STATUS: FOUNDATION DESIGN FOR v0.0.30+
> PURPOSE: Define how Semantic Risks are surfaced to the Boss before generation.

---

# 1. Core Principle

AXON의 목표는 단순 로그 출력 시스템이 아니다.

Boss Board는 다음 역할을 수행해야 한다:

- 단순 실패 보고 ❌
- 의미론적 위험 노출 ✅
- 사용자 결정 유도 ✅
- Sealed IR 생성 지원 ✅

즉:

```text
Error Viewer -> Arbitration Console
```

으로 진화해야 한다.

---

# 2. 현재 방식의 문제점

현재 AXON:

```text
spec.md
 -> architecture.md
 -> tasks
 -> generation
 -> compile
 -> failure
 -> user intervention
```

문제:

- 사용자는 너무 늦게 개입한다.
- 이미 환각 코드가 생성된 이후다.
- 문제 원인을 역추적해야 한다.
- LLM이 빈칸을 이미 상상으로 메운 상태다.

즉:

```text
Post-Failure Intervention
```

구조다.

v0.0.30의 목표는:

```text
Pre-Generation Arbitration
```

이다.

---

# 3. Boss Board의 새로운 역할

Boss Board는 이제:

```text
Semantic Risk Interrupt Console
```

이 되어야 한다.

즉:

- 작업 실패 후 보는 창 ❌
- 생성 전 의미론적 판결 창 ✅

---

# 4. 핵심 UI 구조

Boss Board는 최소 4개의 레이어를 가져야 한다.

---

# 4-1. Semantic Risk Queue

가장 중요하다.

단순 Tasks 리스트가 아니다.

사용자에게:

```text
"무엇이 정의되지 않았는가"
```

를 보여줘야 한다.

예시:

```text
[SEMANTIC INTERRUPTION]

Type:
Data Model Undefined

Location:
database.c

Problem:
struct user_record exists in IR
but field layout is undefined.

Risk:
LLM may hallucinate memory layout.

Required Arbitration:
Define fields and ownership.
```

핵심:

문제 자체를 보여주는 것이 아니라:

```text
"LLM이 어디서 상상하려 하는가"
```

를 보여줘야 한다.

---

# 4-2. Risk Severity

모든 문제를 동일하게 취급하면 안 된다.

Severity 필요:

| Level | 의미 |
|---|---|
| LOW | 자동 처리 가능 |
| MEDIUM | 사용자 검토 권장 |
| HIGH | 생성 중단 |
| CRITICAL | 전체 파이프라인 정지 |

예시:

```text
CRITICAL:
Undefined ownership policy.

HIGH:
Optional dependency escalation.

MEDIUM:
Function naming ambiguity.
```

---

# 4-3. Arbitration Panel

사용자가 즉시 결정 가능해야 한다.

예시:

```text
[DECISION REQUIRED]

Dependency:
ncurses

Current State:
Optional in spec.md
Core in architecture.md

Choose:

[ ] Exclude
[ ] Optional
[ ] Mandatory
```

핵심:

사용자는:

```text
"코드를 수정"
```

하는 것이 아니다.

```text
"법을 결정"
```

하는 것이다.

---

# 4-4. Sealed Result Preview

사용자의 결정이:

```text
IR에 어떻게 봉인되는가
```

를 즉시 보여줘야 한다.

예시:

```json
{
  "dependency": "ncurses",
  "tier": "optional",
  "is_blocking": false,
  "is_sealed": true
}
```

핵심:

Boss는:

```text
"결정 -> 규약화"
```

를 직접 눈으로 봐야 한다.

---

# 5. 왜 Tasks Box만으로는 부족한가

현재 Tasks 시스템은:

```text
무엇을 구현할 것인가
```

를 보여준다.

하지만 v0.0.30은:

```text
무엇이 정의되지 않았는가
```

를 보여줘야 한다.

즉:

```text
Implementation Queue
```

가 아니라:

```text
Semantic Risk Queue
```

가 필요하다.

---

# 6. 새로운 파이프라인

기존:

```text
Spec
 -> Generate
 -> Fail
 -> User
```

새 구조:

```text
Spec
 -> Semantic Debugger
 -> Interrupt
 -> Boss Arbitration
 -> Sealed IR
 -> Generation
 -> Compile
```

핵심 변화:

사용자는:

```text
생성 이후
```

가 아니라:

```text
생성 이전
```

에 개입한다.

---

# 7. 가장 중요한 철학

AXON은:

```text
"창조 엔진"
```

이 아니다.

AXON은:

```text
"결정된 규약을 조립하는 기계"
```

다.

따라서:

```text
No Semantic Closure
 -> No Generation
```

이 최상위 원칙이어야 한다.

---

# 8. KISS 유지 원칙

중요:

Semantic Debugger가:

- 완전한 정적 분석기
- AI 추론 엔진
- 자동 보완 시스템

이 되면 안 된다.

KISS 유지:

```text
모르면 생성하지 않는다.
```

딱 여기까지만 한다.

즉:

- 자동 추론 ❌
- 자동 보완 ❌
- 사용자 판결 요청 ✅

---

# 9. 최종 목표

최종적으로 Boss Board는:

```text
Build Monitor
```

가 아니라:

```text
Semantic Arbitration Console
```

이 되어야 한다.

그리고 AXON의 생성 엔진은:

```text
"추론 기반 생성"
```

이 아니라:

```text
"봉인된 규약 기반 기계적 조립"
```

만 수행해야 한다.

---

# 10. Foundation Note

이 문서는:

```text
Multi-MD 기반 Semantic Sealing 구조
Boss Arbitration 시스템
Semantic Risk Queue UI
```

의 기반 설계 문서다.

향후:

- semantic_debug