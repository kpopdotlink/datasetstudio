//! 토크나이저 모듈

/// 토크나이저 트레이트
pub trait Tokenizer: Send + Sync {
    /// 텍스트의 토큰 수 추정
    fn estimate_tokens(&self, text: &str) -> usize;

    /// 토크나이저 이름
    fn name(&self) -> &str;
}

/// 근사 토크나이저 (문자 기반)
///
/// 평균적으로 영어는 4자당 1토큰, 한국어는 2자당 1토큰으로 추정
pub struct ApproxTokenizer;

impl Default for ApproxTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ApproxTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Tokenizer for ApproxTokenizer {
    fn estimate_tokens(&self, text: &str) -> usize {
        let mut tokens = 0;
        let mut chars_in_word = 0;

        for c in text.chars() {
            if c.is_whitespace() {
                if chars_in_word > 0 {
                    // 영어: ~4자당 1토큰, 한국어: ~2자당 1토큰
                    tokens += (chars_in_word + 3) / 4;
                    chars_in_word = 0;
                }
            } else if is_cjk(c) {
                // CJK 문자는 대략 2자당 1토큰
                if chars_in_word > 0 {
                    tokens += (chars_in_word + 3) / 4;
                    chars_in_word = 0;
                }
                tokens += 1;
            } else {
                chars_in_word += 1;
            }
        }

        if chars_in_word > 0 {
            tokens += (chars_in_word + 3) / 4;
        }

        tokens.max(1)
    }

    fn name(&self) -> &str {
        "approx"
    }
}

/// CJK 문자 판별
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK 통합 한자
        '\u{3400}'..='\u{4DBF}' |   // CJK 통합 한자 확장 A
        '\u{AC00}'..='\u{D7AF}' |   // 한글 음절
        '\u{1100}'..='\u{11FF}' |   // 한글 자모
        '\u{3130}'..='\u{318F}' |   // 한글 호환 자모
        '\u{3040}'..='\u{309F}' |   // 히라가나
        '\u{30A0}'..='\u{30FF}'     // 가타카나
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_tokenizer_english() {
        let tokenizer = ApproxTokenizer::new();

        // 영어 텍스트
        let text = "Hello world";
        let tokens = tokenizer.estimate_tokens(text);
        assert!(tokens >= 2);
    }

    #[test]
    fn test_approx_tokenizer_korean() {
        let tokenizer = ApproxTokenizer::new();

        // 한국어 텍스트
        let text = "안녕하세요 세계";
        let tokens = tokenizer.estimate_tokens(text);
        assert!(tokens >= 2);
    }

    #[test]
    fn test_approx_tokenizer_mixed() {
        let tokenizer = ApproxTokenizer::new();

        // 혼합 텍스트
        let text = "Hello 안녕하세요 World 세계";
        let tokens = tokenizer.estimate_tokens(text);
        assert!(tokens >= 4);
    }
}
