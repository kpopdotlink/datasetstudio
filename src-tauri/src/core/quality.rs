//! 품질 검사 모듈

use serde::{Deserialize, Serialize};

/// 품질 규칙
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRules {
    /// 최소 문자 수
    pub min_chars: Option<usize>,
    /// 최대 문자 수
    pub max_chars: Option<usize>,
    /// 최소 토큰 수
    pub min_tokens: Option<usize>,
    /// 최대 토큰 수
    pub max_tokens: Option<usize>,
    /// 최대 연속 공백 수
    pub max_consecutive_spaces: Option<usize>,
    /// 최대 연속 줄바꿈 수
    pub max_consecutive_newlines: Option<usize>,
    /// 특수문자 비율 제한 (0.0 ~ 1.0)
    pub max_special_char_ratio: Option<f64>,
    /// 중복 문자 비율 제한 (0.0 ~ 1.0)
    pub max_repeated_char_ratio: Option<f64>,
}

impl Default for QualityRules {
    fn default() -> Self {
        Self {
            min_chars: Some(50),
            max_chars: Some(10000),
            min_tokens: Some(10),
            max_tokens: Some(2048),
            max_consecutive_spaces: Some(5),
            max_consecutive_newlines: Some(3),
            max_special_char_ratio: Some(0.3),
            max_repeated_char_ratio: Some(0.5),
        }
    }
}

/// 품질 경고
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWarning {
    pub code: String,
    pub message: String,
    pub severity: WarningSeverity,
}

/// 경고 심각도
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// 품질 검사 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub warnings: Vec<QualityWarning>,
    pub score: f64, // 0.0 ~ 1.0
}

/// 텍스트 품질 검사
pub fn check_quality(text: &str, token_count: usize, rules: &QualityRules) -> QualityCheckResult {
    let mut warnings = Vec::new();
    let char_count = text.chars().count();

    // 최소 문자 수 검사
    if let Some(min) = rules.min_chars {
        if char_count < min {
            warnings.push(QualityWarning {
                code: "TOO_SHORT".to_string(),
                message: format!("텍스트가 너무 짧습니다 ({} < {} 자)", char_count, min),
                severity: WarningSeverity::Warning,
            });
        }
    }

    // 최대 문자 수 검사
    if let Some(max) = rules.max_chars {
        if char_count > max {
            warnings.push(QualityWarning {
                code: "TOO_LONG".to_string(),
                message: format!("텍스트가 너무 깁니다 ({} > {} 자)", char_count, max),
                severity: WarningSeverity::Warning,
            });
        }
    }

    // 최소 토큰 수 검사
    if let Some(min) = rules.min_tokens {
        if token_count < min {
            warnings.push(QualityWarning {
                code: "LOW_TOKENS".to_string(),
                message: format!("토큰 수가 너무 적습니다 ({} < {})", token_count, min),
                severity: WarningSeverity::Warning,
            });
        }
    }

    // 최대 토큰 수 검사
    if let Some(max) = rules.max_tokens {
        if token_count > max {
            warnings.push(QualityWarning {
                code: "HIGH_TOKENS".to_string(),
                message: format!("토큰 수가 너무 많습니다 ({} > {})", token_count, max),
                severity: WarningSeverity::Warning,
            });
        }
    }

    // 연속 공백 검사
    if let Some(max) = rules.max_consecutive_spaces {
        let space_pattern = " ".repeat(max + 1);
        if text.contains(&space_pattern) {
            warnings.push(QualityWarning {
                code: "CONSECUTIVE_SPACES".to_string(),
                message: format!("{}개 이상의 연속 공백이 있습니다", max + 1),
                severity: WarningSeverity::Info,
            });
        }
    }

    // 연속 줄바꿈 검사
    if let Some(max) = rules.max_consecutive_newlines {
        let newline_pattern = "\n".repeat(max + 1);
        if text.contains(&newline_pattern) {
            warnings.push(QualityWarning {
                code: "CONSECUTIVE_NEWLINES".to_string(),
                message: format!("{}개 이상의 연속 줄바꿈이 있습니다", max + 1),
                severity: WarningSeverity::Info,
            });
        }
    }

    // 특수문자 비율 검사
    if let Some(max_ratio) = rules.max_special_char_ratio {
        if char_count > 0 {
            let special_count = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
            let ratio = special_count as f64 / char_count as f64;
            if ratio > max_ratio {
                warnings.push(QualityWarning {
                    code: "HIGH_SPECIAL_CHARS".to_string(),
                    message: format!("특수문자 비율이 높습니다 ({:.1}% > {:.1}%)", ratio * 100.0, max_ratio * 100.0),
                    severity: WarningSeverity::Warning,
                });
            }
        }
    }

    // 중복 문자 비율 검사 (같은 문자가 연속으로 반복되는 경우)
    if let Some(max_ratio) = rules.max_repeated_char_ratio {
        if char_count > 0 {
            let repeated_count = count_repeated_chars(text);
            let ratio = repeated_count as f64 / char_count as f64;
            if ratio > max_ratio {
                warnings.push(QualityWarning {
                    code: "HIGH_REPEATED_CHARS".to_string(),
                    message: format!("중복 문자 비율이 높습니다 ({:.1}% > {:.1}%)", ratio * 100.0, max_ratio * 100.0),
                    severity: WarningSeverity::Warning,
                });
            }
        }
    }

    // 빈 텍스트 검사
    if text.trim().is_empty() {
        warnings.push(QualityWarning {
            code: "EMPTY_TEXT".to_string(),
            message: "텍스트가 비어있습니다".to_string(),
            severity: WarningSeverity::Error,
        });
    }

    // 점수 계산 (경고가 없으면 1.0, 경고가 많을수록 낮아짐)
    let error_count = warnings.iter().filter(|w| matches!(w.severity, WarningSeverity::Error)).count();
    let warning_count = warnings.iter().filter(|w| matches!(w.severity, WarningSeverity::Warning)).count();
    let info_count = warnings.iter().filter(|w| matches!(w.severity, WarningSeverity::Info)).count();

    let score = 1.0 - (error_count as f64 * 0.3 + warning_count as f64 * 0.15 + info_count as f64 * 0.05);
    let score = score.max(0.0).min(1.0);

    QualityCheckResult { warnings, score }
}

/// 연속으로 반복되는 문자 수 계산
fn count_repeated_chars(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut count = 0;
    let mut i = 0;

    while i < chars.len() {
        let mut j = i + 1;
        while j < chars.len() && chars[j] == chars[i] {
            j += 1;
        }
        // 3개 이상 연속되면 카운트
        if j - i >= 3 {
            count += j - i;
        }
        i = j;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_check_empty() {
        let rules = QualityRules::default();
        let result = check_quality("", 0, &rules);
        assert!(result.warnings.iter().any(|w| w.code == "EMPTY_TEXT"));
    }

    #[test]
    fn test_quality_check_short() {
        let rules = QualityRules::default();
        let result = check_quality("Short text", 2, &rules);
        assert!(result.warnings.iter().any(|w| w.code == "TOO_SHORT"));
    }

    #[test]
    fn test_quality_check_good() {
        let rules = QualityRules {
            min_chars: Some(5),
            max_chars: Some(1000),
            min_tokens: Some(1),
            max_tokens: Some(100),
            ..Default::default()
        };
        let result = check_quality("This is a good quality text with enough content.", 10, &rules);
        assert!(result.warnings.is_empty());
        assert_eq!(result.score, 1.0);
    }
}
