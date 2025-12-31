//! Export 프리셋 모듈

use crate::db::DbPool;
use crate::error::Result;
use serde_json::json;

/// 기본 프리셋 확인 및 생성
pub fn ensure_builtin_presets(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;

    // Text-only 프리셋
    conn.execute(
        r#"INSERT OR IGNORE INTO presets (name, schema_version, mapping_json, is_builtin)
           VALUES ('text-only', '1.0', ?1, 1)"#,
        [&json!({"text": "content"}).to_string()],
    )?;

    // Prompt/Completion 프리셋
    conn.execute(
        r#"INSERT OR IGNORE INTO presets (name, schema_version, mapping_json, is_builtin)
           VALUES ('prompt-completion', '1.0', ?1, 1)"#,
        [&json!({"prompt": "prefix", "completion": "content"}).to_string()],
    )?;

    // Instruction/Context/Response 프리셋
    conn.execute(
        r#"INSERT OR IGNORE INTO presets (name, schema_version, mapping_json, is_builtin)
           VALUES ('instruction-context-response', '1.0', ?1, 1)"#,
        [
            &json!({"instruction": "prefix", "context": "content", "response": "suffix"})
                .to_string(),
        ],
    )?;

    // Chat 프리셋 (OpenAI 형식)
    conn.execute(
        r#"INSERT OR IGNORE INTO presets (name, schema_version, mapping_json, validators_json, is_builtin)
           VALUES ('chat', '1.0', ?1, ?2, 1)"#,
        [
            &json!({
                "messages": [
                    {"role": "system", "content": "system_prompt"},
                    {"role": "user", "content": "content"},
                    {"role": "assistant", "content": "response"}
                ]
            }).to_string(),
            &json!({
                "min_messages": 2,
                "required_roles": ["user", "assistant"]
            }).to_string(),
        ],
    )?;

    // ShareGPT 프리셋
    conn.execute(
        r#"INSERT OR IGNORE INTO presets (name, schema_version, mapping_json, is_builtin)
           VALUES ('sharegpt', '1.0', ?1, 1)"#,
        [&json!({
            "conversations": [
                {"from": "human", "value": "content"},
                {"from": "gpt", "value": "response"}
            ]
        }).to_string()],
    )?;

    Ok(())
}
