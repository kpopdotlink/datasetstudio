//! 잡 타입 정의

use crate::core::chunker::ChunkParams;
use crate::core::exporter::ExportConfig;
use serde::{Deserialize, Serialize};

/// 청크 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkFilter {
    /// 특정 문서 ID만 처리
    pub document_ids: Option<Vec<i64>>,
    /// 특정 소스 ID만 처리
    pub source_ids: Option<Vec<i64>>,
    /// 이미 청크된 문서도 다시 처리
    pub rechunk: bool,
}

/// 잡 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    Scan,
    Chunk {
        params: ChunkParams,
        filter: ChunkFilter,
    },
    Export { config: ExportConfig },
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Scan => write!(f, "scan"),
            JobType::Chunk { .. } => write!(f, "chunk"),
            JobType::Export { .. } => write!(f, "export"),
        }
    }
}

/// 잡 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// 잡 이벤트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvent {
    pub job_id: i64,
    pub event_type: JobEventType,
}

/// 잡 이벤트 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobEventType {
    #[serde(rename = "progress")]
    Progress {
        progress: f64,
        message: Option<String>,
    },
    #[serde(rename = "completed")]
    Completed { result: Option<String> },
    #[serde(rename = "failed")]
    Failed { error: String },
}
