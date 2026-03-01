// 数据库模型和操作

use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::models::{Market, ChangePoint};
use super::schema::Database;

/// 存储的价格点
#[derive(Debug, Clone)]
pub struct StoredPricePoint {
    pub market_id: String,
    pub token_id: String,
    pub timestamp: i64,
    pub price: f64,
    pub volume: Option<f64>,
    pub interval: String,
}

/// 市场统计
#[derive(Debug, Clone)]
pub struct MarketStats {
    pub price_count: i64,
    pub first_timestamp: i64,
    pub last_timestamp: i64,
    pub avg_price: f64,
}

impl Database {
    /// 插入或更新市场
    pub fn upsert_market(&self, market: &Market) -> Result<()> {
        let outcomes_json = serde_json::to_string(&market.outcomes)?;
        let tokens_json = market.clob_token_ids.as_ref()
            .map(serde_json::to_string).transpose()?;

        self.execute(|conn| {
            conn.execute(
                "INSERT INTO markets (id, question, condition_id, slug, outcomes, volume, liquidity, active, closed, end_date, category, clob_token_ids)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    question = excluded.question,
                    volume = excluded.volume,
                    liquidity = excluded.liquidity,
                    active = excluded.active,
                    closed = excluded.closed,
                    category = excluded.category,
                    updated_at = strftime('%s', 'now')",
                params![
                    market.id,
                    market.question,
                    market.condition_id,
                    market.slug,
                    outcomes_json,
                    market.volume,
                    market.liquidity,
                    market.active,
                    market.closed,
                    market.end_date,
                    market.category,
                    tokens_json,
                ],
            )?;
            Ok(())
        })
    }

    /// 批量插入价格点
    pub fn insert_price_points(&self, points: &[StoredPricePoint]) -> Result<()> {
        self.execute(|conn| {
            let tx = conn.unchecked_transaction()?;

            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO price_history (market_id, token_id, timestamp, price, volume, interval)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
                )?;

                for point in points {
                    stmt.execute(params![
                        &point.market_id,
                        &point.token_id,
                        point.timestamp,
                        point.price,
                        point.volume,
                        &point.interval,
                    ])?;
                }
            }

            tx.commit()?;
            Ok(())
        })
    }

    /// 插入变化点
    pub fn insert_change_point(&self, market_id: &str, token_id: &str, cp: &ChangePoint) -> Result<i64> {
        self.execute(|conn| {
            conn.execute(
                "INSERT INTO change_points (market_id, token_id, timestamp, price_before, price_after, change_magnitude, confidence, detection_method)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    market_id,
                    token_id,
                    cp.timestamp,
                    cp.price_before,
                    cp.price_after,
                    cp.change_magnitude,
                    cp.confidence,
                    "z_score",
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    /// 获取指定 token 的最后时间戳
    pub fn get_last_timestamp(&self, token_id: &str) -> Result<Option<i64>> {
        self.execute(|conn| {
            let result = conn.query_row(
                "SELECT MAX(timestamp) FROM price_history WHERE token_id = ?1",
                params![token_id],
                |row| row.get(0),
            )?;
            Ok(result)
        })
    }

    /// 获取所有活跃市场
    pub fn get_active_markets(&self) -> Result<Vec<StoredMarket>> {
        self.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, question, condition_id, slug, outcomes, volume, liquidity, active, closed, end_date, category, clob_token_ids
                 FROM markets WHERE active = 1 AND closed = 0"
            )?;

            let markets = stmt.query_map([], |row| {
                Ok(StoredMarket {
                    id: row.get(0)?,
                    question: row.get(1)?,
                    condition_id: row.get(2)?,
                    slug: row.get(3)?,
                    outcomes: row.get(4)?,
                    volume: row.get(5)?,
                    liquidity: row.get(6)?,
                    active: row.get(7)?,
                    closed: row.get(8)?,
                    end_date: row.get(9)?,
                    category: row.get(10)?,
                    clob_token_ids: row.get(11)?,
                })
            })?.collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(markets)
        })
    }

    /// 获取市场统计
    pub fn get_market_stats(&self, market_id: &str) -> Result<MarketStats> {
        self.execute(|conn| {
            let row = conn.query_row(
                "SELECT COUNT(*), MIN(timestamp), MAX(timestamp), AVG(price)
                 FROM price_history WHERE market_id = ?1",
                params![market_id],
                |row| {
                    Ok(MarketStats {
                        price_count: row.get(0)?,
                        first_timestamp: row.get(1)?,
                        last_timestamp: row.get(2)?,
                        avg_price: row.get(3)?,
                    })
                },
            )?;
            Ok(row)
        })
    }
}

/// 存储的市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMarket {
    pub id: String,
    pub question: String,
    pub condition_id: String,
    pub slug: String,
    pub outcomes: String,  // JSON
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub active: bool,
    pub closed: bool,
    pub end_date: Option<String>,  // ISO 8601 日期字符串或 NULL
    pub category: Option<String>,
    pub clob_token_ids: Option<String>,  // JSON
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::schema::Database;

    #[test]
    fn test_upsert_and_get_market() {
        let db = Database::in_memory().unwrap();
        db.init_schema().unwrap();

        let market = Market {
            id: "test-1".to_string(),
            question: "Test question?".to_string(),
            condition_id: "cond-1".to_string(),
            slug: "test-question".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            volume: Some(1000.0),
            liquidity: Some(500.0),
            active: true,
            closed: false,
            end_date: None,
            category: Some("test".to_string()),
            clob_token_ids: Some(vec!["token-1".to_string()]),
        };

        db.upsert_market(&market).unwrap();

        let markets = db.get_active_markets().unwrap();
        assert_eq!(markets.len(), 1);
        assert_eq!(markets[0].id, "test-1");
    }

    #[test]
    fn test_insert_price_points() {
        let db = Database::in_memory().unwrap();
        db.init_schema().unwrap();

        // 首先插入一个市场以满足外键约束
        let market = Market {
            id: "m1".to_string(),
            question: "Test?".to_string(),
            condition_id: "c1".to_string(),
            slug: "test".to_string(),
            outcomes: vec!["Yes".to_string()],
            volume: None,
            liquidity: None,
            active: true,
            closed: false,
            end_date: None,
            category: None,
            clob_token_ids: None,
        };
        db.upsert_market(&market).unwrap();

        let points = vec![
            StoredPricePoint {
                market_id: "m1".to_string(),
                token_id: "t1".to_string(),
                timestamp: 1000,
                price: 0.5,
                volume: Some(100.0),
                interval: "1h".to_string(),
            },
            StoredPricePoint {
                market_id: "m1".to_string(),
                token_id: "t1".to_string(),
                timestamp: 2000,
                price: 0.6,
                volume: Some(150.0),
                interval: "1h".to_string(),
            },
        ];

        db.insert_price_points(&points).unwrap();

        let last_ts = db.get_last_timestamp("t1").unwrap();
        assert_eq!(last_ts, Some(2000));
    }
}
