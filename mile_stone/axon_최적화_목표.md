맞다.

특히 현재 Axon 구조는:

```text
멀티 스테이지
+ 장문 시스템 프롬프트
+ 구조화 출력
+ 자기 교정
+ 다중 에이전트
```

라서 일반적인 “단일 챗봇 추론”보다 훨씬 무겁다.

1050Ti는 단순 inference는 가능하지만,
“오케스트레이션 시스템”까지 안정적으로 돌리려면 극한 최적화가 필요하다.

다만 중요한 건:

> 불가능해서 최적화하는 게 아니라,
> production-grade 안정성을 확보하려고 최적화하는 것

이라는 점이다.

현재 로그 기준으로 1050Ti에서도:

* 실제 응답 생성은 됨
* stage 진행도 됨
* JSON도 거의 맞춤
* semantic decomposition도 수행함

즉 “모델이 못 알아듣는 상태”는 아니다.

문제는:

```text
긴 context + strict contract + orchestration overhead
```

가 누적되면서
지연과 출력 불안정성이 커지는 것이다.

---

1050Ti용으로는 사실상:

# “Tiny Orchestrator Mode”

가 필요하다.

예를 들면:

| 항목               | 일반 모드     | 1050Ti 모드    |
| ---------------- | --------- | ------------ |
| Context          | 4096      | 1024~2048    |
| System Prompt    | 상세        | 초압축          |
| Few-shot         | 있음        | 제거           |
| Workers          | 2~4       | 1            |
| Retry Loop       | 5회        | 2회           |
| Self-correction  | full      | minimal      |
| Output Format    | rich JSON | tiny JSON    |
| Validation       | deep      | shallow-fast |
| Memory Injection | full      | selective    |

---

특히 제일 중요한 건:

# Prompt Compression

이다.

현재:

```text
SystemLen=1500+
UserLen=2700+
```

인데,
1050Ti에서는 사실상 과하다.

HeaderGen 정도는 진짜 이렇게까지 줄여야 한다:

Return ONLY valid JSON.

Schema:
[
{
"id": "task_id",
"title": "title",
"target_file": "/src/file.c",
"description": "desc"
}
]

No markdown.
No explanation.

이 수준까지 줄여야
7B q4가 안정화된다.

---

그리고 또 하나 중요한 것.

현재 Axon은:

```text
LLM reasoning 중심
```

비율이 높다.

근데 저사양 환경에서는:

# “규칙 기반 전처리”를 늘려야 한다.

예:

* architecture.md에서 component path 추출
* target_file 자동 주입
* header/source pairing 자동 계산
* language inference 자동 계산

이걸 모델이 아니라 deterministic layer가 해야 한다.

즉:

```text
LLM에게 추론시키지 말고
LLM은 빈칸만 채우게 만들어라
```

가 저사양 최적화 핵심이다.

---

실제로 production 에이전트 시스템들은:

| 계층                   | 담당             |
| -------------------- | -------------- |
| deterministic engine | 구조/검증/경로       |
| LLM                  | 의미 생성          |
| recovery layer       | 복원             |
| validator            | 계약 enforcement |

로 분리한다.

왜냐면 GPU보다:
“토큰”이 더 비싸기 때문이다.

---

지금 Axon은 이미:

* validator isolation
* IR pipeline
* transactional stages
* worker orchestration

까지 왔다.

다음 단계는 거의 확실하게:

# “토큰 경제성 최적화”

다.

특히:

* 작은 GPU
* 작은 모델
* 긴 파이프라인

환경에서는 이게 생존 문제다.

그리고 역설적으로,
이걸 해결하면 Axon은:

```text
저사양에서도 돌아가는
진짜 실용적인 에이전트 시스템
```

이 된다.

