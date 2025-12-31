---
id: 0016
version: v0.1.1
title: 통계 대시보드
owner: frontend
status: OPEN
priority: P1
milestone: M1
depends_on:
  - "0005"
  - "0015"
acceptance_criteria:
  - "문서/청크/승인률 KPI가 표시된다."
  - "길이 분포 및 태그별 통계가 제공된다."
  - "필터 변경 시 성능 저하가 없다."
---

## 작업 범위
- 통계 API 정의
- 대시보드 UI 구성
- 필터/그래프 연동

## 검증
- KPI 값 정확성 확인
- 대용량 프로젝트 성능 확인
