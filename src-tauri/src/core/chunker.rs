//! 청크 생성 모듈

use crate::commands::chunk::ChunkPreview;
use crate::core::normalize::{normalize_text, NormalizeOptions};
use crate::core::tokenizer::{ApproxTokenizer, Tokenizer};
use crate::db::DbPool;
use crate::error::Result;
use encoding_rs;
use serde::{Deserialize, Serialize};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

/// 길이 단위
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum LengthUnit {
    #[default]
    Character,
    Token,
}

/// 청크 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkParams {
    /// 최대 길이
    pub max_len: usize,
    /// 오버랩 길이
    pub overlap_len: usize,
    /// 길이 단위
    #[serde(default)]
    pub length_unit: LengthUnit,
    /// 문단 경계 유지
    #[serde(default = "default_true")]
    pub preserve_paragraph: bool,
    /// 문장 경계 유지
    #[serde(default = "default_true")]
    pub preserve_sentence: bool,
    /// 하드 컷 허용
    #[serde(default = "default_true")]
    pub allow_hard_cut: bool,
    /// 정규화 옵션
    #[serde(default)]
    pub normalize: NormalizeOptions,
}

fn default_true() -> bool {
    true
}

impl Default for ChunkParams {
    fn default() -> Self {
        Self {
            max_len: 2000,
            overlap_len: 200,
            length_unit: LengthUnit::Character,
            preserve_paragraph: true,
            preserve_sentence: true,
            allow_hard_cut: true,
            normalize: NormalizeOptions::default(),
        }
    }
}

/// 청크 결과
#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub index: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub text: String,
    pub char_count: usize,
    pub token_est: usize,
    pub overlap_prev: usize,
    pub overlap_next: usize,
    pub is_hard_cut: bool,
}

/// 세그먼트 (문단 또는 문장)
#[derive(Debug, Clone)]
struct Segment {
    text: String,
    start: usize,
    end: usize,
}

/// 청크 생성기
pub struct Chunker<'a> {
    params: &'a ChunkParams,
    tokenizer: Box<dyn Tokenizer>,
}

impl<'a> Chunker<'a> {
    pub fn new(params: &'a ChunkParams) -> Self {
        Self {
            params,
            tokenizer: Box::new(ApproxTokenizer::new()),
        }
    }

    /// 텍스트를 청크로 분할
    pub fn chunk(&self, text: &str) -> Vec<ChunkResult> {
        // 파라미터 검증
        if self.params.overlap_len >= self.params.max_len {
            return vec![];
        }

        // 정규화
        let normalized = normalize_text(text, &self.params.normalize);

        // 세그먼트 분할
        let segments = self.split_segments(&normalized);

        if segments.is_empty() {
            return vec![];
        }

        // 청크 생성
        self.build_chunks(&normalized, &segments)
    }

    /// 세그먼트 분할 (문단 → 문장)
    fn split_segments(&self, text: &str) -> Vec<Segment> {
        let mut segments = Vec::new();

        if self.params.preserve_paragraph {
            // 문단 분할 (빈 줄 기준)
            let mut start = 0;
            let mut in_paragraph = false;
            let mut para_start = 0;

            for line in text.lines() {
                if line.trim().is_empty() {
                    if in_paragraph {
                        let para_text = text[para_start..start].to_string();
                        if !para_text.trim().is_empty() {
                            if self.params.preserve_sentence {
                                segments.extend(self.split_sentences(&para_text, para_start));
                            } else {
                                segments.push(Segment {
                                    text: para_text,
                                    start: para_start,
                                    end: start,
                                });
                            }
                        }
                        in_paragraph = false;
                    }
                } else if !in_paragraph {
                    para_start = start;
                    in_paragraph = true;
                }

                start += line.len() + 1; // +1 for newline
            }

            // 마지막 문단
            if in_paragraph && para_start < text.len() {
                let para_text = text[para_start..].to_string();
                if !para_text.trim().is_empty() {
                    if self.params.preserve_sentence {
                        segments.extend(self.split_sentences(&para_text, para_start));
                    } else {
                        segments.push(Segment {
                            text: para_text,
                            start: para_start,
                            end: text.len(),
                        });
                    }
                }
            }
        } else if self.params.preserve_sentence {
            segments = self.split_sentences(text, 0);
        } else {
            // 전체를 하나의 세그먼트로
            segments.push(Segment {
                text: text.to_string(),
                start: 0,
                end: text.len(),
            });
        }

        segments
    }

    /// 문장 분할
    fn split_sentences(&self, text: &str, offset: usize) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut current_start = 0;

        for sentence in text.unicode_sentences() {
            let start = text[current_start..].find(sentence).unwrap_or(0) + current_start;
            let end = start + sentence.len();

            segments.push(Segment {
                text: sentence.to_string(),
                start: offset + start,
                end: offset + end,
            });

            current_start = end;
        }

        // 남은 텍스트 처리
        if current_start < text.len() {
            let remaining = text[current_start..].to_string();
            if !remaining.trim().is_empty() {
                segments.push(Segment {
                    text: remaining,
                    start: offset + current_start,
                    end: offset + text.len(),
                });
            }
        }

        segments
    }

    /// 청크 빌드
    fn build_chunks(&self, _text: &str, segments: &[Segment]) -> Vec<ChunkResult> {
        let mut chunks = Vec::new();
        let stride = self.params.max_len - self.params.overlap_len;
        let mut seg_idx = 0;
        let mut chunk_idx = 0;

        while seg_idx < segments.len() {
            let mut chunk_text = String::new();
            let mut chunk_len = 0;
            let chunk_start = segments[seg_idx].start;
            let mut chunk_end = chunk_start;
            let first_seg = seg_idx;
            let mut is_hard_cut = false;

            // 세그먼트 수집
            while seg_idx < segments.len() {
                let seg = &segments[seg_idx];
                let seg_len = self.measure(&seg.text);

                if chunk_len + seg_len <= self.params.max_len {
                    if !chunk_text.is_empty() && !seg.text.starts_with(' ') {
                        chunk_text.push(' ');
                    }
                    chunk_text.push_str(&seg.text);
                    chunk_len += seg_len;
                    chunk_end = seg.end;
                    seg_idx += 1;
                } else if chunk_len == 0 {
                    // 세그먼트가 max_len보다 큼 → 하드 컷
                    if self.params.allow_hard_cut {
                        let cut_text = self.hard_cut(&seg.text);
                        chunk_text = cut_text;
                        chunk_len = self.measure(&chunk_text);
                        chunk_end = chunk_start + chunk_text.len();
                        is_hard_cut = true;

                        // 세그먼트를 분할
                        // 다음 반복에서 나머지 처리를 위해 seg_idx는 증가시키지 않음
                        break;
                    } else {
                        // 하드 컷 불허 → 건너뜀
                        seg_idx += 1;
                        continue;
                    }
                } else {
                    break;
                }
            }

            if chunk_text.is_empty() {
                continue;
            }

            // 오버랩 계산
            let overlap_prev = if chunk_idx > 0 {
                self.params.overlap_len.min(chunk_len)
            } else {
                0
            };

            let overlap_next = if seg_idx < segments.len() {
                self.params.overlap_len
            } else {
                0
            };

            chunks.push(ChunkResult {
                index: chunk_idx,
                start_offset: chunk_start,
                end_offset: chunk_end,
                text: chunk_text.clone(),
                char_count: chunk_text.chars().count(),
                token_est: self.tokenizer.estimate_tokens(&chunk_text),
                overlap_prev,
                overlap_next,
                is_hard_cut,
            });

            chunk_idx += 1;

            // 오버랩을 위해 세그먼트 인덱스 되감기
            if !is_hard_cut && seg_idx < segments.len() {
                let _target_len = stride;
                let mut rewind_len = 0;
                let mut new_seg_idx = seg_idx;

                while new_seg_idx > first_seg {
                    new_seg_idx -= 1;
                    rewind_len += self.measure(&segments[new_seg_idx].text);
                    if rewind_len >= self.params.overlap_len {
                        break;
                    }
                }

                if new_seg_idx < seg_idx {
                    seg_idx = new_seg_idx + 1;
                }
            }
        }

        chunks
    }

    /// 길이 측정
    fn measure(&self, text: &str) -> usize {
        match self.params.length_unit {
            LengthUnit::Character => text.chars().count(),
            LengthUnit::Token => self.tokenizer.estimate_tokens(text),
        }
    }

    /// 하드 컷
    fn hard_cut(&self, text: &str) -> String {
        match self.params.length_unit {
            LengthUnit::Character => text.chars().take(self.params.max_len).collect(),
            LengthUnit::Token => {
                // 토큰 기반 컷은 근사적으로 처리
                let chars: Vec<char> = text.chars().collect();
                let mut end = chars.len();

                while end > 0 {
                    let slice: String = chars[..end].iter().collect();
                    if self.tokenizer.estimate_tokens(&slice) <= self.params.max_len {
                        return slice;
                    }
                    end -= 1;
                }

                String::new()
            }
        }
    }
}

/// 파일을 인코딩 자동 감지하여 읽기
fn read_file_with_encoding(path: &str) -> Result<String> {
    let bytes = std::fs::read(path)?;

    // BOM 확인
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        return Ok(String::from_utf8_lossy(&bytes[3..]).to_string());
    }

    // UTF-8 시도
    if let Ok(text) = std::str::from_utf8(&bytes) {
        return Ok(text.to_string());
    }

    // EUC-KR/CP949 시도 (한국어 파일)
    let (text, _, _) = encoding_rs::EUC_KR.decode(&bytes);
    Ok(text.to_string())
}

/// 청크 미리보기 생성
pub fn generate_preview(
    pool: &DbPool,
    project_path: &Path,
    doc_id: i64,
    params: &ChunkParams,
    sample_size: usize,
) -> Result<Vec<ChunkPreview>> {
    let conn = pool.get()?;

    // 문서 조회
    let (original_path, checksum): (Option<String>, String) = conn.query_row(
        "SELECT original_path, checksum FROM documents WHERE id = ?1",
        [doc_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // 텍스트 읽기 (인코딩 자동 감지)
    let text = if let Some(path) = original_path {
        read_file_with_encoding(&path)?
    } else {
        // 캐시에서 읽기 (캐시는 UTF-8로 저장됨)
        let cache_path = project_path.join("cache").join(&checksum);
        std::fs::read_to_string(&cache_path)?
    };

    // 청크 생성
    let chunker = Chunker::new(params);
    let chunks = chunker.chunk(&text);

    // 샘플 크기만큼 반환
    let previews: Vec<ChunkPreview> = chunks
        .into_iter()
        .take(sample_size)
        .map(|c| ChunkPreview {
            index: c.index,
            text: c.text,
            char_count: c.char_count,
            token_est: c.token_est,
            overlap_prev: c.overlap_prev,
            overlap_next: c.overlap_next,
            is_hard_cut: c.is_hard_cut,
        })
        .collect();

    Ok(previews)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let params = ChunkParams {
            max_len: 100,
            overlap_len: 20,
            ..Default::default()
        };

        let text = "This is a test. This is another sentence. And one more sentence here.";
        let chunker = Chunker::new(&params);
        let chunks = chunker.chunk(text);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.char_count <= params.max_len);
        }
    }

    #[test]
    fn test_determinism() {
        let params = ChunkParams {
            max_len: 100,
            overlap_len: 20,
            ..Default::default()
        };

        let text =
            "This is a test document with multiple sentences. It should be chunked consistently.";
        let chunker = Chunker::new(&params);

        let chunks1 = chunker.chunk(text);
        let chunks2 = chunker.chunk(text);

        assert_eq!(chunks1.len(), chunks2.len());
        for (c1, c2) in chunks1.iter().zip(chunks2.iter()) {
            assert_eq!(c1.text, c2.text);
            assert_eq!(c1.start_offset, c2.start_offset);
            assert_eq!(c1.end_offset, c2.end_offset);
        }
    }

    #[test]
    fn test_overlap_validation() {
        let params = ChunkParams {
            max_len: 100,
            overlap_len: 150, // overlap >= max_len
            ..Default::default()
        };

        let text = "Test text";
        let chunker = Chunker::new(&params);
        let chunks = chunker.chunk(text);

        assert!(chunks.is_empty());
    }
}
