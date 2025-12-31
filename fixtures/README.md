# Test Fixtures

이 디렉토리에는 테스트 및 개발에 사용되는 샘플 데이터가 포함되어 있습니다.

## 파일 목록

| 파일 | 설명 | 용도 |
|------|------|------|
| `sample_short.txt` | 한국어 짧은 텍스트 (~500자) | 기본 청크 테스트 |
| `sample_medium.txt` | 한국어 중간 길이 텍스트 (~3000자) | 청크 분할/오버랩 테스트 |
| `sample_english.md` | 영어 마크다운 (~3000자) | 마크다운 처리 및 인코딩 테스트 |

## 사용법

### Rust 테스트

```rust
#[cfg(test)]
mod tests {
    const SAMPLE_SHORT: &str = include_str!("../../../fixtures/sample_short.txt");

    #[test]
    fn test_with_sample_data() {
        let chunks = chunk(SAMPLE_SHORT, &params);
        assert!(!chunks.is_empty());
    }
}
```

### 수동 테스트

1. 앱에서 "소스 추가" 선택
2. `fixtures` 폴더를 드래그 앤 드롭
3. 청크 생성 및 리뷰 테스트

## 저작권

모든 샘플 텍스트는 테스트 목적으로 직접 작성되었으며, 저작권 문제가 없습니다.
