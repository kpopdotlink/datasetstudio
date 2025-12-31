---
id: 0006
version: v0.1.1
title: 프로젝트 생성/열기 및 상태 복원
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0003"
  - "0005"
acceptance_criteria:
  - "project.json이 생성되고 기본 설정이 기록된다."
  - "프로젝트 폴더 구조가 자동 생성된다."
  - "앱 재시작 후 프로젝트 상태가 복원된다."
---

## 작업 범위
- 프로젝트 생성/열기 플로우
- 기본 폴더 구조 생성
- 상태 복원 로직

## 검증
- 새 프로젝트 생성 후 재시작 복원 확인
- project.json 설정 일관성 확인
