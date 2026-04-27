# 📄 AXON v0.0.17 Spec

## User-Controlled Local/Cloud Orchestration

------------------------------------------------------------------------

## 0. 🎯 Goal

### Primary Goal

사용자가 선택한 로컬/클라우드 LLM 구성을 기반으로\
**검증 가능하고 예측 가능한 오케스트레이션 시스템 구축**

### Success Criteria

-   사용자 설정 그대로 실행 (자동 변경 없음)
-   잘못된 설정에 대한 명확한 경고
-   로컬/클라우드 동일 인터페이스
-   병목 상황에서도 제어 가능 상태 유지
-   실시간 상태 가시화

------------------------------------------------------------------------

## 1. 🧠 Core Concept

### Before

    LLM = 자동 배치되는 분산 노드

### After

    LLM = 사용자가 명시적으로 배치한 실행 에이전트

> AXON은 결정하지 않는다. 실행하고 검증한다.

------------------------------------------------------------------------

## 2. 🏗️ Architecture

    [AXON Daemon]
        ↓
    [Agent Registry]
        ↓
    [Execution Engine]
        ↓
    [Runtime (Local / Cloud)]

------------------------------------------------------------------------

## 3. ⚙️ Configuration

### axon_config.json

``` json
{
  "agents": {
    "architect": {
      "runtime": "local",
      "endpoint": "http://127.0.0.1:11434",
      "model": "mistral:latest"
    },
    "seniors": [
      {
        "runtime": "cloud",
        "provider": "gemini",
        "model": "gemini-2.5-flash"
      }
    ],
    "juniors": [
      {
        "runtime": "local",
        "endpoint": "http://127.0.0.1:11434",
        "model": "llama3.2:1b"
      }
    ]
  },
  "execution": {
    "review_queue_limit": 5,
    "sampling_rate": 0.3,
    "fallback_enabled": true
  }
}
```

------------------------------------------------------------------------

## 4. 🔍 Validation Phase

### Local

    GET /api/tags

### Cloud

-   API Key 확인
-   Ping 테스트

### Output

    [OK] model available
    [WARN] model mismatch
    [FAIL] endpoint unreachable

------------------------------------------------------------------------

## 5. 🧪 Capability Profiling (Optional)

### Tests

-   Code Generation
-   Reasoning
-   Latency

### Example Output

    mistral:latest
    - latency: 1800ms
    - reasoning: medium
    - code: medium

------------------------------------------------------------------------

## 6. 🎭 Role Assignment

-   100% 사용자 정의
-   시스템은 검증만 수행

```{=html}
<!-- -->
```
    [WARN] Senior assigned to low reasoning model

------------------------------------------------------------------------

## 7. 🧱 Bootstrapping

### Context Injection

    Current Agents:
    - Architect: Local Mistral
    - Senior: Cloud Gemini
    - Junior: Local Llama

------------------------------------------------------------------------

## 8. ⚙️ Execution Flow

    Task
     ↓
    Junior
     ↓
    Senior
     ↓
    Architect
     ↓
    Commit

### Agent Selection

``` rust
round_robin(role_agents)
```

------------------------------------------------------------------------

## 9. 🔁 Progressive Validation

### Sampling Review

``` rust
if random() < sampling_rate {
    review()
}
```

### Retry Limit

``` rust
if retry_count > 3 {
    escalate_to_architect()
}
```

------------------------------------------------------------------------

## 10. 🚦 Bottleneck Control

### Queue Limit

``` rust
if review_queue.len() > limit {
    pause_junior_dispatch();
}
```

### Adaptive Throttle

``` rust
delay = last_latency * 1.2
```

------------------------------------------------------------------------

## 11. 🔌 Failure Handling

``` rust
if agent_unavailable {
    skip_agent();
}
```

------------------------------------------------------------------------

## 12. 📊 Observability

-   latency
-   queue 상태
-   실패율
-   역할 매핑

------------------------------------------------------------------------

## 13. 📖 Scenarios

### Local Only

    Architect: Mistral
    Senior: Mistral
    Junior: Llama

### Mixed

    Senior: Cloud
    Junior: Local

### Bottleneck

    Junior 20 / Senior 1

------------------------------------------------------------------------

## 14. 🚧 Scope

### Included

-   Explicit config
-   Validation
-   Execution control
-   Observability

### Excluded

-   Auto role assignment
-   Auto scaling
-   Auto model selection

------------------------------------------------------------------------

## Conclusion

    User → 결정
    AXON → 실행
    AXON → 검증 / 가시화
