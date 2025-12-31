---
id: 0011
version: v0.1.1
title: Export 엔진 및 manifest
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0010"
  - "0008"
acceptance_criteria:
  - "프리셋 3종이 JSONL로 출력된다."
  - "파일 분할과 manifest 생성이 동작한다."
  - "Export Run 이력이 저장된다."
---

## 작업 범위
- 프리셋 정의/매핑 규칙
- JSONL 직렬화 및 분할
- export_runs/export_files 기록

## 검증
- 동일 조건에서 결정적 순서 확인
- manifest 파라미터 일관성 확인
