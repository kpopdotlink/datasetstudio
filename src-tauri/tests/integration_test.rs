//! 통합 테스트

use std::path::PathBuf;

/// fixtures 경로 반환
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures")
}

#[test]
fn test_fixtures_exist() {
    let fixtures = fixtures_path();
    assert!(fixtures.exists(), "fixtures 디렉토리가 존재해야 함");
    assert!(
        fixtures.join("sample_short.txt").exists(),
        "sample_short.txt가 존재해야 함"
    );
    assert!(
        fixtures.join("sample_medium.txt").exists(),
        "sample_medium.txt가 존재해야 함"
    );
    assert!(
        fixtures.join("sample_english.md").exists(),
        "sample_english.md가 존재해야 함"
    );
}

#[test]
fn test_read_fixtures() {
    let fixtures = fixtures_path();

    let short_content = std::fs::read_to_string(fixtures.join("sample_short.txt"))
        .expect("sample_short.txt 읽기 실패");
    assert!(
        short_content.contains("인공지능"),
        "한국어 콘텐츠 포함 확인"
    );

    let medium_content = std::fs::read_to_string(fixtures.join("sample_medium.txt"))
        .expect("sample_medium.txt 읽기 실패");
    assert!(
        medium_content.contains("소프트웨어"),
        "한국어 콘텐츠 포함 확인"
    );

    let english_content = std::fs::read_to_string(fixtures.join("sample_english.md"))
        .expect("sample_english.md 읽기 실패");
    assert!(
        english_content.contains("Machine Learning"),
        "영어 콘텐츠 포함 확인"
    );
}

#[test]
fn test_utf8_encoding() {
    let fixtures = fixtures_path();

    // 모든 fixture 파일이 유효한 UTF-8인지 확인
    for file in &["sample_short.txt", "sample_medium.txt", "sample_english.md"] {
        let content = std::fs::read(fixtures.join(file)).expect("파일 읽기 실패");
        let text = String::from_utf8(content);
        assert!(text.is_ok(), "{} 파일이 유효한 UTF-8이어야 함", file);
    }
}
