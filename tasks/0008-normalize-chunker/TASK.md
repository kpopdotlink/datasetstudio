---
id: 0008
version: v0.1.1
title: 정규화 및 청크 엔진
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0007"
  - "0005"
acceptance_criteria:
  - "max_len/overlap_len 규칙이 적용된다."
  - "문단/문장 경계를 우선으로 유지한다."
  - "동일 입력에서 동일 청크가 생성된다."
---

## 작업 범위
- 정규화 규칙 구현
- 문단/문장 기반 분절
- overlap 포함 청크 생성

## 검증
- 결정성(Deterministic) 테스트
- 경계 규칙 우선순위 확인
