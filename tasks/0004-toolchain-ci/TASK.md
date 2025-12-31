---
id: 0004
version: v0.1.1
title: 툴체인 고정 및 CI 파이프라인 구성
owner: release
status: OPEN
priority: P0
milestone: M0
depends_on:
  - "0003"
acceptance_criteria:
  - "Rust/Node 버전이 고정된다."
  - "GitHub Actions에서 macOS/Windows 빌드가 통과한다."
  - "락파일 정책이 명확히 정리된다."
---

## 작업 범위
- Rust toolchain 고정
- 패키지 매니저 및 락파일 정책 정의
- GitHub Actions 빌드 파이프라인 구성

## 검증
- CI 빌드 성공 로그 확인
- 로컬에서 빌드 재현성 확인
