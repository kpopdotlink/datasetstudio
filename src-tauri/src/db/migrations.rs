//! 데이터베이스 마이그레이션

use crate::error::Result;
use rusqlite::Connection;

/// 초기 마이그레이션 SQL
const MIGRATION_0001_INITIAL: &str = include_str!("../../migrations/0001_initial.sql");
const MIGRATION_0002_MANUAL_ENTRIES: &str = include_str!("../../migrations/0002_manual_entries.sql");

/// 모든 마이그레이션 목록
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_initial", MIGRATION_0001_INITIAL),
    ("0002_manual_entries", MIGRATION_0002_MANUAL_ENTRIES),
];

/// 모든 마이그레이션 실행
pub fn run_all(conn: &Connection) -> Result<()> {
    // 마이그레이션 추적 테이블 생성
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 각 마이그레이션 실행
    for (name, sql) in MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM _migrations WHERE name = ?1",
                [name],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !applied {
            log::info!("마이그레이션 실행: {}", name);
            conn.execute_batch(sql)?;

            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
            log::info!("마이그레이션 완료: {}", name);
        }
    }

    Ok(())
}
