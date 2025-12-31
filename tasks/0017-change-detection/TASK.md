---
id: 0017
version: v0.1.1
title: 문서 변경 감지 및 충돌 처리
owner: core
status: OPEN
priority: P1
milestone: M1
depends_on:
  - "0007"
acceptance_criteria:
  - "수동 새로고침이 기본으로 동작한다."
  - "변경 감지 시 선택 UI가 제공된다."
  - "새 버전 생성/무시/덮어쓰기 정책이 기록된다."
---

## 작업 범위
- 변경 감지 정책 구현
- 충돌 처리 UX 구성
- 문서 버전 정책 기록

## 검증
- 변경 파일 감지/처리 흐름 확인
- 정책 선택에 따른 결과 확인
