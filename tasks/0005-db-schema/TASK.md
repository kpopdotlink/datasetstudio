---
id: 0005
version: v0.1.1
title: SQLite 스키마 및 마이그레이션
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0003"
acceptance_criteria:
  - "PRD 스키마를 반영한 테이블/인덱스가 생성된다."
  - "마이그레이션이 idempotent 하게 동작한다."
  - "FTS 확장 고려 구조가 포함된다(P1 준비)."
---

## 작업 범위
- 핵심 테이블/인덱스 정의
- 마이그레이션 도입

## 검증
- 신규 프로젝트에서 DB 생성 확인
- 기존 DB에 마이그레이션 적용 확인
