---
id: 0012
version: v0.1.1
title: 보안/권한/경로 스코프
owner: core
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0003"
acceptance_criteria:
  - "프로젝트 루트 기반 스코프가 적용된다."
  - "네트워크 접근이 기본 비활성이다."
  - "위험 작업에 확인 단계가 있다."
---

## 작업 범위
- Tauri 권한/스코프 설정
- 네트워크 접근 기본 비활성화
- 위험 작업 UX 확인 단계

## 검증
- 허용된 경로 외 접근 차단 확인
- 네트워크 비활성 기본값 확인
