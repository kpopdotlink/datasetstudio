//! 데이터베이스 모델 정의

use serde::{Deserialize, Serialize};

/// 소스
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: i64,
    #[serde(rename = "type")]
    pub source_type: String,
    pub path_or_key: String,
    pub display_name: Option<String>,
    pub added_at: String,
    pub meta_json: Option<String>,
}

/// 문서
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub source_id: i64,
    pub display_name: String,
    pub original_path: Option<String>,
    pub checksum: String,
    pub encoding: String,
    pub status: String,
    pub byte_size: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// 청크 버전
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkVersion {
    pub id: i64,
    pub params_json: String,
    pub tokenizer_id: Option<String>,
    pub created_at: String,
}

/// 청크
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: i64,
    pub document_id: i64,
    pub chunk_version_id: i64,
    pub chunk_index: i64,
    pub start_offset: i64,
    pub end_offset: i64,
    pub text_cached: Option<String>,
    pub char_count: i64,
    pub token_est: i64,
    pub overlap_prev: i64,
    pub overlap_next: i64,
    pub review_status: String,
    pub is_hard_cut: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 청크 편집
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkEdit {
    pub id: i64,
    pub chunk_id: i64,
    pub edited_text: String,
    pub edited_at: String,
    pub edit_meta_json: Option<String>,
}

/// 태그
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub key: String,
    pub value: String,
}

/// 프리셋
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: i64,
    pub name: String,
    pub schema_version: String,
    pub mapping_json: String,
    pub validators_json: Option<String>,
    pub is_builtin: bool,
    pub created_at: String,
}

/// Export 실행 기록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRun {
    pub id: i64,
    pub preset_id: Option<i64>,
    pub preset_name: String,
    pub params_json: String,
    pub filters_json: String,
    pub stats_json: Option<String>,
    pub created_at: String,
}

/// Export 파일
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub id: i64,
    pub export_run_id: i64,
    pub filename: String,
    pub size_bytes: i64,
    pub line_count: i64,
    pub sha256: Option<String>,
}

/// 잡
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub job_type: String,
    pub status: String,
    pub progress: f64,
    pub payload_json: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 매뉴얼 엔트리 (SFT 데이터셋용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualEntry {
    pub id: i64,
    pub preset_id: i64,
    pub data_json: String,
    pub review_status: String,
    pub created_at: String,
    pub updated_at: String,
}
