---
id: 0009
version: v0.1.1
title: 백그라운드 잡 및 청크 미리보기
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0008"
acceptance_criteria:
  - "job progress/취소가 UI에 전달된다."
  - "미리보기 결과가 본 생성 결과와 동일하다."
  - "대용량 처리 시 UI가 응답성을 유지한다."
---

## 작업 범위
- 잡 큐/진행률 이벤트
- 청크 미리보기 API
- 취소/중단 처리

## 검증
- 100MB 이상 입력에서 UI 응답성 확인
- 미리보기와 실제 생성 결과 비교
