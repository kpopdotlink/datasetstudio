//! Export 엔진 모듈

use crate::commands::export::ExportFilters;
use crate::db::models::Preset;
use crate::db::DbPool;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Export 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub preset_id: i64,
    pub filters: ExportFilters,
    pub max_file_size_mb: usize,
}

/// Export 매니페스트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub export_id: String,
    pub preset: String,
    pub params: Value,
    pub filters: ExportFilters,
    pub stats: ExportStats,
    pub files: Vec<ExportFileInfo>,
}

/// Export 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub total_lines: usize,
    pub total_chars: usize,
    pub token_est_total: usize,
}

/// Export 파일 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFileInfo {
    pub name: String,
    pub bytes: usize,
    pub lines: usize,
}

/// Exportable 청크
pub struct ExportableChunk {
    pub id: i64,
    pub text: String,
    pub edited_text: Option<String>,
    pub document_name: String,
    pub source_id: i64,
    pub source_name: String,
    pub tags: Vec<(String, String)>,
}

/// Exportable 수동 입력 엔트리
pub struct ExportableManualEntry {
    pub id: i64,
    pub data_json: String,
}

/// JSONL Export 실행
pub fn export_to_jsonl(
    pool: &DbPool,
    project_path: &Path,
    config: &ExportConfig,
) -> Result<ExportManifest> {
    let conn = pool.get()?;

    // 프리셋 조회
    let preset: Preset = conn.query_row(
        "SELECT id, name, schema_version, mapping_json, validators_json, is_builtin, created_at
         FROM presets WHERE id = ?1",
        [config.preset_id],
        |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                schema_version: row.get(2)?,
                mapping_json: row.get(3)?,
                validators_json: row.get(4)?,
                is_builtin: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )?;

    // 매핑 파싱
    let mapping: Value = serde_json::from_str(&preset.mapping_json)?;

    // 청크 조회 SQL 구성
    let mut sql = String::from(
        "SELECT c.id, COALESCE(ce.edited_text, c.text_cached) as text, d.display_name,
                d.source_id, COALESCE(s.display_name, s.path_or_key) as source_name
         FROM chunks c
         JOIN documents d ON c.document_id = d.id
         JOIN sources s ON d.source_id = s.id
         LEFT JOIN (
             SELECT chunk_id, edited_text
             FROM chunk_edits
             WHERE id IN (SELECT MAX(id) FROM chunk_edits GROUP BY chunk_id)
         ) ce ON c.id = ce.chunk_id
         WHERE 1=1",
    );

    if config.filters.approved_only {
        sql.push_str(" AND c.review_status = 'approved'");
    }

    if let Some(ref doc_ids) = config.filters.document_ids {
        if !doc_ids.is_empty() {
            let ids: Vec<String> = doc_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(" AND c.document_id IN ({})", ids.join(",")));
        }
    }

    if let Some(ref source_ids) = config.filters.source_ids {
        if !source_ids.is_empty() {
            let ids: Vec<String> = source_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(" AND d.source_id IN ({})", ids.join(",")));
        }
    }

    sql.push_str(" ORDER BY d.source_id, d.display_name, c.chunk_index");

    // 청크 조회
    let mut stmt = conn.prepare(&sql)?;
    let chunks: Vec<ExportableChunk> = stmt
        .query_map([], |row| {
            Ok(ExportableChunk {
                id: row.get(0)?,
                text: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                edited_text: None,
                document_name: row.get(2)?,
                source_id: row.get(3)?,
                source_name: row.get(4)?,
                tags: vec![],
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // 수동 입력 엔트리 조회 (include_manual_entries가 true일 때)
    let manual_entries: Vec<ExportableManualEntry> =
        if config.filters.include_manual_entries.unwrap_or(true) {
            let manual_sql = if config.filters.approved_only {
                "SELECT id, data_json FROM manual_entries WHERE preset_id = ?1 AND review_status = 'approved' ORDER BY id"
            } else {
                "SELECT id, data_json FROM manual_entries WHERE preset_id = ?1 AND review_status != 'rejected' ORDER BY id"
            };
            let mut manual_stmt = conn.prepare(manual_sql)?;
            let entries: Vec<ExportableManualEntry> = manual_stmt
                .query_map([config.preset_id], |row| {
                    Ok(ExportableManualEntry {
                        id: row.get(0)?,
                        data_json: row.get(1)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            entries
        } else {
            Vec::new()
        };

    // Export 실행
    let export_id = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let exports_dir = project_path.join("exports").join(&export_id);
    fs::create_dir_all(&exports_dir)?;

    let max_file_bytes = config.max_file_size_mb * 1024 * 1024;
    let split_by_source = config.filters.split_by_source.unwrap_or(false);
    let mut files: Vec<ExportFileInfo> = Vec::new();
    let mut total_lines = 0;
    let mut total_chars = 0;
    let mut total_tokens = 0;

    let mut current_file_num = 1;
    let mut current_file_bytes = 0;
    let mut current_file_lines = 0;
    let mut current_source_id: Option<i64> = None;
    let mut current_source_name: Option<String> = None;
    let mut writer: Option<BufWriter<File>> = None;

    // 파일명 생성 헬퍼
    let make_filename = |source_name: &Option<String>, file_num: usize| -> String {
        if let Some(ref name) = source_name {
            // 소스별 분리 시 소스명 사용 (파일명에 안전한 문자만 사용)
            let safe_name: String = name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .collect();
            format!("{}.jsonl", safe_name)
        } else {
            format!("export_{:04}.jsonl", file_num)
        }
    };

    for chunk in chunks {
        // JSON 라인 생성
        let json_line = create_json_line(&chunk, &mapping)?;
        let line_bytes = json_line.as_bytes();
        let line_len = line_bytes.len() + 1; // +1 for newline

        // 소스가 바뀌었는지 확인 (split_by_source 모드)
        let source_changed = split_by_source && current_source_id != Some(chunk.source_id);

        // 새 파일 필요 여부 확인
        if writer.is_none() || source_changed || current_file_bytes + line_len > max_file_bytes {
            // 이전 파일 닫기
            if let Some(mut w) = writer.take() {
                w.flush()?;
                let filename = if split_by_source {
                    make_filename(&current_source_name, current_file_num)
                } else {
                    format!("export_{:04}.jsonl", current_file_num)
                };
                files.push(ExportFileInfo {
                    name: filename,
                    bytes: current_file_bytes,
                    lines: current_file_lines,
                });
                current_file_num += 1;
            }

            // 소스 정보 업데이트
            if split_by_source {
                current_source_id = Some(chunk.source_id);
                current_source_name = Some(chunk.source_name.clone());
            }

            // 새 파일 열기
            let filename = if split_by_source {
                make_filename(&current_source_name, current_file_num)
            } else {
                format!("export_{:04}.jsonl", current_file_num)
            };
            let file_path = exports_dir.join(&filename);
            let file = File::create(&file_path)?;
            writer = Some(BufWriter::new(file));
            current_file_bytes = 0;
            current_file_lines = 0;
        }

        // 라인 쓰기
        if let Some(ref mut w) = writer {
            w.write_all(line_bytes)?;
            w.write_all(b"\n")?;
            current_file_bytes += line_len;
            current_file_lines += 1;
            total_lines += 1;
            total_chars += chunk.text.chars().count();
            total_tokens += estimate_tokens(&chunk.text);
        }
    }

    // 수동 입력 엔트리 쓰기
    for entry in manual_entries {
        // data_json은 이미 JSON 객체이므로 그대로 사용
        let json_line = &entry.data_json;
        let line_bytes = json_line.as_bytes();
        let line_len = line_bytes.len() + 1; // +1 for newline

        // 새 파일 필요 여부 확인
        if writer.is_none() || current_file_bytes + line_len > max_file_bytes {
            // 이전 파일 닫기
            if let Some(mut w) = writer.take() {
                w.flush()?;
                files.push(ExportFileInfo {
                    name: format!("export_{:04}.jsonl", current_file_num),
                    bytes: current_file_bytes,
                    lines: current_file_lines,
                });
                current_file_num += 1;
            }

            // 새 파일 열기
            let file_path = exports_dir.join(format!("export_{:04}.jsonl", current_file_num));
            let file = File::create(&file_path)?;
            writer = Some(BufWriter::new(file));
            current_file_bytes = 0;
            current_file_lines = 0;
        }

        // 라인 쓰기
        if let Some(ref mut w) = writer {
            w.write_all(line_bytes)?;
            w.write_all(b"\n")?;
            current_file_bytes += line_len;
            current_file_lines += 1;
            total_lines += 1;
            total_chars += entry.data_json.chars().count();
            total_tokens += estimate_tokens(&entry.data_json);
        }
    }

    // 마지막 파일 닫기
    if let Some(mut w) = writer.take() {
        w.flush()?;
        if current_file_lines > 0 {
            let filename = if split_by_source {
                make_filename(&current_source_name, current_file_num)
            } else {
                format!("export_{:04}.jsonl", current_file_num)
            };
            files.push(ExportFileInfo {
                name: filename,
                bytes: current_file_bytes,
                lines: current_file_lines,
            });
        }
    }

    // 매니페스트 생성
    let manifest = ExportManifest {
        export_id: export_id.clone(),
        preset: preset.name,
        params: json!({}),
        filters: config.filters.clone(),
        stats: ExportStats {
            total_lines,
            total_chars,
            token_est_total: total_tokens,
        },
        files,
    };

    // 매니페스트 저장
    let manifest_path = exports_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    // DB에 기록
    conn.execute(
        "INSERT INTO export_runs (preset_id, preset_name, params_json, filters_json, stats_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            config.preset_id,
            manifest.preset,
            "{}",
            serde_json::to_string(&config.filters)?,
            serde_json::to_string(&manifest.stats)?
        ],
    )?;

    let export_run_id = conn.last_insert_rowid();

    for file_info in &manifest.files {
        conn.execute(
            "INSERT INTO export_files (export_run_id, filename, size_bytes, line_count)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                export_run_id,
                file_info.name,
                file_info.bytes,
                file_info.lines
            ],
        )?;
    }

    Ok(manifest)
}

/// JSON 라인 생성
fn create_json_line(chunk: &ExportableChunk, mapping: &Value) -> Result<String> {
    let text = chunk.text.clone();

    let output = if let Some(obj) = mapping.as_object() {
        // messages 배열이 있으면 Chat 형식
        if let Some(messages) = obj.get("messages") {
            if let Some(msgs) = messages.as_array() {
                let mut result_msgs = Vec::new();
                for msg in msgs {
                    if let Some(msg_obj) = msg.as_object() {
                        let mut new_msg = serde_json::Map::new();
                        for (key, value) in msg_obj {
                            if key == "content" {
                                // content 필드에 실제 텍스트 삽입
                                let content_type = value.as_str().unwrap_or("content");
                                let content = match content_type {
                                    "content" => text.clone(),
                                    "system_prompt" => String::from("You are a helpful assistant."),
                                    "response" => String::new(), // 응답은 빈 문자열
                                    _ => text.clone(),
                                };
                                new_msg.insert(key.clone(), json!(content));
                            } else {
                                // role 등 다른 필드는 그대로 사용
                                new_msg.insert(key.clone(), value.clone());
                            }
                        }
                        result_msgs.push(Value::Object(new_msg));
                    }
                }
                json!({ "messages": result_msgs })
            } else {
                json!({"text": text})
            }
        }
        // conversations 배열이 있으면 ShareGPT 형식
        else if let Some(conversations) = obj.get("conversations") {
            if let Some(convs) = conversations.as_array() {
                let mut result_convs = Vec::new();
                for conv in convs {
                    if let Some(conv_obj) = conv.as_object() {
                        let mut new_conv = serde_json::Map::new();
                        for (key, value) in conv_obj {
                            if key == "value" {
                                let value_type = value.as_str().unwrap_or("content");
                                let content = match value_type {
                                    "content" => text.clone(),
                                    "response" => String::new(),
                                    _ => text.clone(),
                                };
                                new_conv.insert(key.clone(), json!(content));
                            } else {
                                new_conv.insert(key.clone(), value.clone());
                            }
                        }
                        result_convs.push(Value::Object(new_conv));
                    }
                }
                json!({ "conversations": result_convs })
            } else {
                json!({"text": text})
            }
        }
        // 일반 객체 형식
        else {
            let mut result = serde_json::Map::new();
            for (key, value) in obj {
                let content = match value.as_str() {
                    Some("content") => text.clone(),
                    Some("prefix") => format!("{}", chunk.document_name),
                    Some("suffix") => String::new(),
                    _ => text.clone(),
                };
                result.insert(key.clone(), json!(content));
            }
            Value::Object(result)
        }
    } else {
        // 기본: text-only
        json!({"text": text})
    };

    Ok(serde_json::to_string(&output)?)
}

/// 토큰 추정 (간단한 근사)
fn estimate_tokens(text: &str) -> usize {
    // 대략 4자당 1토큰
    (text.chars().count() + 3) / 4
}
