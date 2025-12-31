---
id: 0015
version: v0.1.1
title: 태그/검색/FTS 인덱스
owner: core
status: OPEN
priority: P1
milestone: M1
depends_on:
  - "0005"
  - "0007"
acceptance_criteria:
  - "태그 CRUD가 동작한다."
  - "문서/청크 검색이 가능하다."
  - "FTS 인덱스가 생성된다."
---

## 작업 범위
- 태그 데이터 모델/연결
- 검색 API 및 UI 연동
- FTS 인덱스 관리

## 검증
- 태그 필터 적용 확인
- FTS 검색 성능 확인
