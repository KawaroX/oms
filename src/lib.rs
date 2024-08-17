use rusqlite::params;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use dirs;
use serde_json::error;
use std::path::PathBuf;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use thiserror::Error;
use log::{info, error};

#[derive(Error, Debug)]
pub enum OriginMonitorError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Pool error: {0}")]
    PoolError(#[from] r2d2::Error),
    #[error("Initialization error: {0}")]
    InitError(String),
}

type Result<T> = std::result::Result<T, OriginMonitorError>;


// ===========数据库初始化============ //

lazy_static! {
    static ref DB_PATH: PathBuf = get_database_path();
    static ref POOL: Pool<SqliteConnectionManager> = create_connection_pool();
}

fn get_database_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("Could not find local data directory")
        .join("Luwav")
        .join("oms.db")
}

fn create_connection_pool() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(DB_PATH.as_path());
    Pool::new(manager).expect("Failed to create pool")
}

// ================================== //

pub struct OriginMonitor {
    pub pool: Pool<SqliteConnectionManager>,
}

impl OriginMonitor {
    pub fn new() -> Result<Self> {
        let pool = POOL.clone();
        let conn = pool.get()?;
        
        conn.execute_batch("
            BEGIN TRANSACTION;
            CREATE TABLE IF NOT EXISTS origins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                clusters_id JSON NOT NULL DEFAULT '[]',
                UNIQUE(name)
            );
            CREATE TABLE IF NOT EXISTS clusters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                origin_id INTEGER NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(origin_id) REFERENCES origins(id),
                UNIQUE(name, origin_id)
            );
            CREATE TABLE IF NOT EXISTS waves (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                cluster_id INTEGER NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                tags JSON NOT NULL DEFAULT '[]',
                preview TEXT NOT NULL DEFAULT '',
                content JSON NOT NULL DEFAULT '[]',
                FOREIGN KEY(cluster_id) REFERENCES clusters(id),
                UNIQUE(name, cluster_id)
            );
            CREATE INDEX IF NOT EXISTS idx_clusters_origin_id ON clusters(origin_id);
            CREATE INDEX IF NOT EXISTS idx_waves_cluster_id ON waves(cluster_id);
            CREATE INDEX IF NOT EXISTS idx_waves_content ON waves((json_extract(content, '$.text')));
            COMMIT;
        ").map_err(|e| OriginMonitorError::InitError(format!("Failed to initialize database: {}", e)))?;

        info!("Database initialized successfully");
        Ok(OriginMonitor { pool })
    }

    pub fn create_origin(&self, name: &str) -> Result<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO origins (name) VALUES (?)",
            params![name],
        )?;
        let id = conn.last_insert_rowid();
        info!("Created new origin with id: {}", id);
        Ok(id)
    }

    // 明天再写捏
}
