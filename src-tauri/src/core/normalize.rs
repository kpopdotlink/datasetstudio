//! 텍스트 정규화 모듈

use serde::{Deserialize, Serialize};

/// 줄바꿈 모드
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
    Native,
}

/// 정규화 옵션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizeOptions {
    /// 줄바꿈 모드
    #[serde(default)]
    pub line_ending: LineEnding,
    /// 연속 공백 축소
    #[serde(default)]
    pub collapse_whitespace: bool,
    /// 탭을 스페이스로 변환 (None이면 변환 안 함)
    #[serde(default)]
    pub tab_to_spaces: Option<usize>,
    /// 각 줄 앞뒤 공백 제거
    #[serde(default)]
    pub trim_lines: bool,
    /// 연속 빈 줄 축소 (최대 N개)
    #[serde(default)]
    pub max_consecutive_newlines: Option<usize>,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            line_ending: LineEnding::Lf,
            collapse_whitespace: false,
            tab_to_spaces: None,
            trim_lines: false,
            max_consecutive_newlines: None,
        }
    }
}

/// 텍스트 정규화
pub fn normalize_text(input: &str, opts: &NormalizeOptions) -> String {
    let mut result = input.to_string();

    // 1. 줄바꿈 통일
    result = result.replace("\r\n", "\n").replace('\r', "\n");

    // 2. 탭 변환
    if let Some(spaces) = opts.tab_to_spaces {
        let space_str = " ".repeat(spaces);
        result = result.replace('\t', &space_str);
    }

    // 3. 줄 단위 처리
    let lines: Vec<String> = result
        .lines()
        .map(|line| {
            let mut l = line.to_string();

            // 줄 앞뒤 공백 제거
            if opts.trim_lines {
                l = l.trim().to_string();
            }

            // 연속 공백 축소
            if opts.collapse_whitespace {
                l = collapse_spaces(&l);
            }

            l
        })
        .collect();

    // 4. 연속 빈 줄 처리
    let lines = if let Some(max) = opts.max_consecutive_newlines {
        collapse_newlines(&lines, max)
    } else {
        lines
    };

    // 5. 줄바꿈 문자 결정
    let newline = match opts.line_ending {
        LineEnding::Lf => "\n",
        LineEnding::Crlf => "\r\n",
        LineEnding::Native => {
            #[cfg(windows)]
            {
                "\r\n"
            }
            #[cfg(not(windows))]
            {
                "\n"
            }
        }
    };

    lines.join(newline)
}

/// 연속 공백을 단일 공백으로 축소
fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;

    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(c);
                prev_space = true;
            }
        } else {
            result.push(c);
            prev_space = false;
        }
    }

    result
}

/// 연속 빈 줄 축소
fn collapse_newlines(lines: &[String], max: usize) -> Vec<String> {
    let mut result = Vec::with_capacity(lines.len());
    let mut empty_count = 0;

    for line in lines {
        if line.is_empty() {
            empty_count += 1;
            if empty_count <= max {
                result.push(line.clone());
            }
        } else {
            empty_count = 0;
            result.push(line.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_line_endings() {
        let input = "Hello\r\nWorld\rTest\n";
        let opts = NormalizeOptions::default();
        let result = normalize_text(input, &opts);
        assert!(!result.contains('\r'));
    }

    #[test]
    fn test_normalize_tabs() {
        let input = "Hello\tWorld";
        let opts = NormalizeOptions {
            tab_to_spaces: Some(4),
            ..Default::default()
        };
        let result = normalize_text(input, &opts);
        assert_eq!(result, "Hello    World");
    }

    #[test]
    fn test_collapse_whitespace() {
        let input = "Hello    World";
        let opts = NormalizeOptions {
            collapse_whitespace: true,
            ..Default::default()
        };
        let result = normalize_text(input, &opts);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_trim_lines() {
        let input = "  Hello  \n  World  ";
        let opts = NormalizeOptions {
            trim_lines: true,
            ..Default::default()
        };
        let result = normalize_text(input, &opts);
        assert_eq!(result, "Hello\nWorld");
    }

    #[test]
    fn test_collapse_newlines() {
        let input = "Hello\n\n\n\nWorld";
        let opts = NormalizeOptions {
            max_consecutive_newlines: Some(1),
            ..Default::default()
        };
        let result = normalize_text(input, &opts);
        assert_eq!(result, "Hello\n\nWorld");
    }
}
