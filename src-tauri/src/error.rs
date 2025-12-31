//! 에러 타입 정의

use thiserror::Error;

/// Dataset Studio 에러 타입
#[derive(Error, Debug)]
pub enum AppError {
    #[error("데이터베이스 오류: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("R2D2 풀 오류: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("IO 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 오류: {0}")]
    Json(#[from] serde_json::Error),

    #[error("디렉토리 탐색 오류: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("프로젝트가 열려있지 않습니다")]
    NoProject,

    #[error("프로젝트가 이미 존재합니다: {0}")]
    ProjectExists(String),

    #[error("유효하지 않은 프로젝트 경로: {0}")]
    InvalidProjectPath(String),

    #[error("문서를 찾을 수 없습니다: {0}")]
    DocumentNotFound(i64),

    #[error("청크를 찾을 수 없습니다: {0}")]
    ChunkNotFound(i64),

    #[error("프리셋을 찾을 수 없습니다: {0}")]
    PresetNotFound(String),

    #[error("잡을 찾을 수 없습니다: {0}")]
    JobNotFound(i64),

    #[error("잡이 이미 실행 중입니다")]
    JobAlreadyRunning,

    #[error("잡이 취소되었습니다")]
    JobCancelled,

    #[error("인코딩 오류: {0}")]
    Encoding(String),

    #[error("유효하지 않은 파라미터: {0}")]
    InvalidParameter(String),

    #[error("내부 오류: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result 타입 별칭
pub type Result<T> = std::result::Result<T, AppError>;
