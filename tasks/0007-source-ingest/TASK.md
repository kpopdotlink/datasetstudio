---
id: 0007
version: v0.1.1
title: 소스 등록 및 문서 관리
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0006"
acceptance_criteria:
  - "폴더/파일 등록이 동작한다."
  - "직접 입력 텍스트 문서가 추가된다."
  - "중복 등록이 차단되고 상태가 부여된다."
---

## 작업 범위
- 폴더/파일 스캔 및 문서 등록
- 인코딩 처리 및 체크섬
- 직접 입력 텍스트 등록

## 검증
- 동일 파일 중복 등록 차단 확인
- 문서 상태가 올바르게 표시됨
