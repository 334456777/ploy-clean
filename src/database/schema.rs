// 数据库模式和连接管理

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 数据库连接
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 打开数据库连接
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // 启用 WAL 模式以提高并发性能
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "cache_size", -64000)?; // 64MB cache

        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    /// 创建内存数据库 (用于测试)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    /// 执行 schema 初始化
    pub fn init_schema(&self) -> Result<()> {
        let schema = include_str!("../../migrations/schema.sql");
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(schema)?;
        Ok(())
    }

    /// 执行操作
    pub fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::in_memory().unwrap();
        db.init_schema().unwrap();

        // 验证表已创建
        let table_count: i64 = db
            .execute(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert!(table_count >= 4); // markets, price_history, change_points, sampling_plans
    }
}
