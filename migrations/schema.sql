-- Poly-Cleaner 数据库模式
-- SQLite WAL 模式配置
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64MB cache
PRAGMA foreign_keys = ON;

-- 市场表
CREATE TABLE IF NOT EXISTS markets (
    id TEXT PRIMARY KEY,
    question TEXT NOT NULL,
    condition_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    outcomes TEXT NOT NULL,  -- JSON array
    volume REAL,
    liquidity REAL,
    active BOOLEAN NOT NULL DEFAULT 1,
    closed BOOLEAN NOT NULL DEFAULT 0,
    end_date TEXT,  -- ISO 8601 日期字符串
    category TEXT,
    clob_token_ids TEXT,  -- JSON array
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 价格历史表
CREATE TABLE IF NOT EXISTS price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    price REAL NOT NULL,
    volume REAL,
    interval TEXT NOT NULL,  -- '1m', '5m', '1h', '1d'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);

-- 变化点表
CREATE TABLE IF NOT EXISTS change_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    price_before REAL,
    price_after REAL,
    change_magnitude REAL,
    confidence REAL,
    detection_method TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);

-- 采样计划表
CREATE TABLE IF NOT EXISTS sampling_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    start_timestamp INTEGER NOT NULL,
    end_timestamp INTEGER NOT NULL,
    interval_seconds INTEGER NOT NULL,
    status TEXT NOT NULL,  -- 'pending', 'active', 'completed', 'failed'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_price_market_timestamp ON price_history(market_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_price_token_timestamp ON price_history(token_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_price_timestamp ON price_history(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_change_points_timestamp ON change_points(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_change_points_market ON change_points(market_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sampling_plans_status ON sampling_plans(status, start_timestamp);
CREATE INDEX IF NOT EXISTS idx_markets_active ON markets(active, closed);
