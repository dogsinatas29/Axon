# AXON: 자동화 소프트웨어 공장 (Phase 07)

[![English](https://img.shields.io/badge/lang-English-blue.svg)](README.md)
[![한국어](https://img.shields.io/badge/lang-한국어-red.svg)](#)

![AXON Concept](https://raw.githubusercontent.com/dogsinatas29/Axon/master/axon%EA%B0%9C%EB%85%90.png)

AXON은 아키텍처 사양을 100% 물리적 무결성을 갖춘 프로덕션 코드로 변환하기 위해 설계된 고성능 결정론적 AI 에이전트 공장입니다.

## 📑 목차
- [🧠 핵심 철학](#-핵심-철학)
- [🏛️ 시스템 아키텍처: 물리 검증 파이프라인](#-시스템-아키텍처-물리-검증-파이프라인-v0023)
- [🏗️ 역할 정의](#-역할-정의)
- [🛠️ 시작하기](#-시작하기)
- [📋 릴리즈 노트](#-릴리즈-노트)

## 🧠 핵심 철학: "아키텍처의 결과물로서의 코드"
AXON은 코딩을 창의적인 글쓰기가 아닌, **결정론적 구체화(Deterministic Materialization)** 과정으로 취급합니다.
- **SSOT (단일 진실 공급원)**: 아키텍처 IR이 곧 법입니다.
- **물리적 무결성**: 코드는 논리적일 뿐만 아니라 물리적 환경(파일 시스템, 런타임)에서도 반드시 생존해야 합니다.
- **대립적 거버넌스**: 에이전트들은 최적의 로직을 생산하기 위해 서로 비판하고 토론(Debate)해야 합니다.

## 🏛️ 시스템 아키텍처: 물리 검증 파이프라인 (v0.0.23+)

![AXON Architecture Concept](asset/mermaid-diagram.png)

AXON Phase 07은 **"낙관적 자동화, 비관적 개입(Optimistic Automation, Pessimistic Intervention)"** 전략을 구현합니다:

1. **논리 승인 (Axon Pass)**: 주니어의 제안서가 논리적 일관성을 갖췄는지 검증합니다.
2. **물리적 배포 (Materialization)**: 코드를 실제 프로젝트 파일 시스템에 작성합니다.
3. **물리 검증 (Harness v0.1)**: 파일 무결성(F1/F2), 진입점(F3), 부작용(F9) 등을 자동 전수 조사합니다.
4. **시니어 게이트 (Final Lock-in)**: 시니어 에이전트가 *실제로 배포된* 실물 코드를 최종 승인합니다.
5. **자동 롤백 (Auto-Rollback)**: 3단계 또는 4단계에서 실패 발생 시, 즉시 이전 상태로 원복하여 공장의 청결을 유지합니다.

### 👴 시니어 개입 시점의 변화
시니어는 이제 **최종 문지기(Final Gatekeeper)** 역할을 수행합니다. 코드가 물리적 환경에서 실행 가능하다는 것이 증명된 *후에* 최종 심사를 진행합니다. 물리 단계에서 실패가 발생하면 시니어에게 즉시 알림이 전송되어 개입이 이루어집니다.

## 🏗️ 역할 정의

### 👑 1. 아키텍트 (Architect / CTO)
- **역할**: 전략적 기획 및 시스템 전체 설계.
- **책임**: 마스터 아키텍처를 생성하고 이를 원자 단위의 태스크로 분해합니다.

### 👴 2. 시니어 (Senior / Tech Lead)
- **역할**: 품질 보증 및 엄격한 코드 리뷰.
- **책임**: 주니어의 제안을 승인하거나 반려하며, '최종 문지기' 규칙을 집행합니다.

### 👶 3. 주니어 (Junior / Developer)
- **역할**: 순수 구현 및 코딩.
- **책임**: 아키텍트의 가이드에 따라 소스 코드와 변경 사항(Diff)을 제출합니다.

## 🛠️ 시작하기

```bash
# 공장 빌드
cargo build --release

# 사양서와 함께 실행
./target/release/axon-daemon run GEMINI.md
```

---
*Antigravity AI 코딩 어시스턴트 제작.*
