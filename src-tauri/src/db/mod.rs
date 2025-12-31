//! 데이터베이스 모듈

pub mod migrations;
pub mod models;
pub mod repository;
pub mod schema;

use crate::error::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// 데이터베이스 연결 풀 타입
pub type DbPool = Pool<SqliteConnectionManager>;

/// 데이터베이스 연결 풀 생성
pub fn create_pool(db_path: &Path) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA cache_size = -64000;",
        )?;
        Ok(())
    });

    let pool = Pool::builder().max_size(10).build(manager)?;

    Ok(pool)
}

/// 마이그레이션 실행
pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;
    migrations::run_all(&conn)?;
    Ok(())
}
