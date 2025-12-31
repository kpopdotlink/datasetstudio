//! 소스 수집 모듈

use crate::commands::source::ScanResult;
use crate::db::models::Source;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use encoding_rs::Encoding;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 소스 타입
#[derive(Debug, Clone, Copy)]
pub enum SourceType {
    Folder,
    File,
    Manual,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Folder => write!(f, "folder"),
            SourceType::File => write!(f, "file"),
            SourceType::Manual => write!(f, "manual"),
        }
    }
}

/// 지원하는 확장자
const SUPPORTED_EXTENSIONS: &[&str] = &["txt", "md"];

/// 소스 추가
pub fn add_source(
    pool: &DbPool,
    project_path: &Path,
    source_type: SourceType,
    path_or_content: &str,
    display_name: Option<String>,
) -> Result<Source> {
    let conn = pool.get()?;

    // 중복 확인
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM sources WHERE path_or_key = ?1",
            [path_or_content],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if exists {
        return Err(AppError::InvalidParameter(format!(
            "이미 등록된 소스: {}",
            path_or_content
        )));
    }

    let name = display_name.unwrap_or_else(|| {
        Path::new(path_or_content)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_or_content.to_string())
    });

    conn.execute(
        "INSERT INTO sources (type, path_or_key, display_name) VALUES (?1, ?2, ?3)",
        rusqlite::params![source_type.to_string(), path_or_content, name],
    )?;

    let id = conn.last_insert_rowid();

    // Manual 타입은 바로 문서로 추가
    if matches!(source_type, SourceType::Manual) {
        let checksum = blake3::hash(path_or_content.as_bytes())
            .to_hex()
            .to_string();

        // 캐시에 저장
        let cache_path = project_path.join("cache").join(&checksum);
        fs::create_dir_all(project_path.join("cache"))?;
        fs::write(&cache_path, path_or_content)?;

        conn.execute(
            "INSERT INTO documents (source_id, display_name, checksum, byte_size, status)
             VALUES (?1, ?2, ?3, ?4, 'unprocessed')",
            rusqlite::params![id, name, checksum, path_or_content.len()],
        )?;
    }

    Ok(Source {
        id,
        source_type: source_type.to_string(),
        path_or_key: path_or_content.to_string(),
        display_name: Some(name),
        added_at: chrono::Utc::now().to_rfc3339(),
        meta_json: None,
    })
}

/// 모든 소스 스캔
pub fn scan_all_sources(pool: &DbPool, project_path: &Path) -> Result<ScanResult> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare("SELECT id, type, path_or_key FROM sources")?;
    let sources: Vec<(i64, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut total_files = 0;
    let mut new_files = 0;
    let mut skipped_files = 0;
    let mut errors = Vec::new();

    for (source_id, source_type, path_or_key) in sources {
        match source_type.as_str() {
            "folder" => {
                let result = scan_folder(pool, project_path, source_id, &path_or_key);
                match result {
                    Ok((t, n, s)) => {
                        total_files += t;
                        new_files += n;
                        skipped_files += s;
                    }
                    Err(e) => errors.push(format!("{}: {}", path_or_key, e)),
                }
            }
            "file" => {
                let result = scan_file(pool, project_path, source_id, &path_or_key);
                match result {
                    Ok(added) => {
                        total_files += 1;
                        if added {
                            new_files += 1;
                        } else {
                            skipped_files += 1;
                        }
                    }
                    Err(e) => errors.push(format!("{}: {}", path_or_key, e)),
                }
            }
            _ => {} // manual은 이미 처리됨
        }
    }

    Ok(ScanResult {
        total_files,
        new_files,
        skipped_files,
        errors,
    })
}

/// 폴더 스캔
fn scan_folder(
    pool: &DbPool,
    project_path: &Path,
    source_id: i64,
    folder_path: &str,
) -> Result<(usize, usize, usize)> {
    let mut total = 0;
    let mut new = 0;
    let mut skipped = 0;

    for entry in WalkDir::new(folder_path).follow_links(true) {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        if let Some(ext) = ext {
            if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                total += 1;
                let path_str = path.to_string_lossy().to_string();
                match scan_file(pool, project_path, source_id, &path_str) {
                    Ok(added) => {
                        if added {
                            new += 1;
                        } else {
                            skipped += 1;
                        }
                    }
                    Err(_) => skipped += 1,
                }
            }
        }
    }

    Ok((total, new, skipped))
}

/// 파일 스캔
fn scan_file(pool: &DbPool, _project_path: &Path, source_id: i64, file_path: &str) -> Result<bool> {
    let conn = pool.get()?;

    // 파일 읽기 및 인코딩 감지
    let bytes = fs::read(file_path)?;
    let (text, encoding) = detect_and_decode(&bytes)?;

    // 체크섬 계산
    let checksum = blake3::hash(text.as_bytes()).to_hex().to_string();

    // 중복 확인
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM documents WHERE source_id = ?1 AND checksum = ?2",
            rusqlite::params![source_id, checksum],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if exists {
        return Ok(false);
    }

    // 파일명
    let display_name = Path::new(file_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    // 문서 추가
    conn.execute(
        "INSERT INTO documents (source_id, display_name, original_path, checksum, encoding, byte_size, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'unprocessed')",
        rusqlite::params![
            source_id,
            display_name,
            file_path,
            checksum,
            encoding,
            bytes.len()
        ],
    )?;

    Ok(true)
}

/// 단일 소스 재스캔
pub fn rescan_source(pool: &DbPool, project_path: &Path, source_id: i64) -> Result<ScanResult> {
    let conn = pool.get()?;

    let (source_type, path_or_key): (String, String) = conn.query_row(
        "SELECT type, path_or_key FROM sources WHERE id = ?1",
        [source_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let mut total_files = 0;
    let mut new_files = 0;
    let mut skipped_files = 0;
    let mut errors = Vec::new();

    match source_type.as_str() {
        "folder" => {
            let result = scan_folder(pool, project_path, source_id, &path_or_key);
            match result {
                Ok((t, n, s)) => {
                    total_files = t;
                    new_files = n;
                    skipped_files = s;
                }
                Err(e) => errors.push(format!("{}: {}", path_or_key, e)),
            }
        }
        "file" => {
            let result = scan_file(pool, project_path, source_id, &path_or_key);
            match result {
                Ok(added) => {
                    total_files = 1;
                    if added {
                        new_files = 1;
                    } else {
                        skipped_files = 1;
                    }
                }
                Err(e) => errors.push(format!("{}: {}", path_or_key, e)),
            }
        }
        _ => {} // manual은 스캔 불필요
    }

    Ok(ScanResult {
        total_files,
        new_files,
        skipped_files,
        errors,
    })
}

/// 문서 변경 정보
#[derive(Debug, Clone)]
pub struct DocumentChange {
    pub document_id: i64,
    pub display_name: String,
    pub original_path: String,
    pub old_checksum: String,
    pub new_checksum: String,
    pub change_type: ChangeType,
}

/// 변경 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// 파일 내용이 변경됨
    Modified,
    /// 파일이 삭제됨
    Deleted,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Modified => write!(f, "modified"),
            ChangeType::Deleted => write!(f, "deleted"),
        }
    }
}

/// 문서 변경 감지
pub fn check_document_changes(pool: &DbPool) -> Result<Vec<DocumentChange>> {
    let conn = pool.get()?;
    let mut changes = Vec::new();

    // 파일 기반 문서만 조회 (manual 타입 제외)
    let mut stmt = conn.prepare(
        "SELECT d.id, d.display_name, d.original_path, d.checksum
         FROM documents d
         JOIN sources s ON d.source_id = s.id
         WHERE s.type != 'manual' AND d.original_path IS NOT NULL"
    )?;

    let documents: Vec<(i64, String, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    for (doc_id, display_name, original_path, old_checksum) in documents {
        let path = std::path::Path::new(&original_path);

        if !path.exists() {
            // 파일이 삭제됨
            changes.push(DocumentChange {
                document_id: doc_id,
                display_name,
                original_path,
                old_checksum,
                new_checksum: String::new(),
                change_type: ChangeType::Deleted,
            });
            continue;
        }

        // 파일 내용 확인
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok((text, _)) = detect_and_decode(&bytes) {
                let new_checksum = blake3::hash(text.as_bytes()).to_hex().to_string();

                if new_checksum != old_checksum {
                    changes.push(DocumentChange {
                        document_id: doc_id,
                        display_name,
                        original_path,
                        old_checksum,
                        new_checksum,
                        change_type: ChangeType::Modified,
                    });
                }
            }
        }
    }

    Ok(changes)
}

/// 문서 변경 적용 (다시 청크 처리 필요 표시)
pub fn apply_document_change(pool: &DbPool, document_id: i64, action: &str) -> Result<()> {
    let conn = pool.get()?;

    match action {
        "rechunk" => {
            // 문서 상태를 needs_attention으로 변경하고 기존 청크 삭제
            conn.execute(
                "UPDATE documents SET status = 'needs_attention', updated_at = datetime('now') WHERE id = ?1",
                [document_id],
            )?;
            conn.execute("DELETE FROM chunks WHERE document_id = ?1", [document_id])?;
        }
        "update_checksum" => {
            // 파일의 새 체크섬으로 업데이트 (변경사항 무시)
            let original_path: String = conn.query_row(
                "SELECT original_path FROM documents WHERE id = ?1",
                [document_id],
                |row| row.get(0),
            )?;

            if let Ok(bytes) = std::fs::read(&original_path) {
                if let Ok((text, _)) = detect_and_decode(&bytes) {
                    let new_checksum = blake3::hash(text.as_bytes()).to_hex().to_string();
                    conn.execute(
                        "UPDATE documents SET checksum = ?1, updated_at = datetime('now') WHERE id = ?2",
                        rusqlite::params![new_checksum, document_id],
                    )?;
                }
            }
        }
        "remove" => {
            // 문서 삭제
            conn.execute("DELETE FROM documents WHERE id = ?1", [document_id])?;
        }
        _ => return Err(AppError::InvalidParameter(format!("Unknown action: {}", action))),
    }

    Ok(())
}

/// 인코딩 감지 및 디코딩
fn detect_and_decode(bytes: &[u8]) -> Result<(String, String)> {
    // BOM 확인
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM
        return Ok((
            String::from_utf8_lossy(&bytes[3..]).to_string(),
            "UTF-8".to_string(),
        ));
    }

    // UTF-8 시도
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok((text.to_string(), "UTF-8".to_string()));
    }

    // encoding_rs로 감지
    let (encoding, _) = Encoding::for_bom(bytes).unwrap_or((encoding_rs::UTF_8, 0));
    let (text, _, had_errors) = encoding.decode(bytes);

    if had_errors {
        // EUC-KR 시도
        let (text, _, _) = encoding_rs::EUC_KR.decode(bytes);
        return Ok((text.to_string(), "EUC-KR".to_string()));
    }

    Ok((text.to_string(), encoding.name().to_string()))
}
