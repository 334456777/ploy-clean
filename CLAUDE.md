# Poly-Cleaner - AI 开发指南

## 项目概述

Poly-Cleaner 是一个基于 Rust 的高性能 Polymarket 数据 ETL 工具，实现基于概率变化程度的自适应采样。

**核心功能**：
- 从 Polymarket/Gamma API 获取预测市场数据
- 自适应采样算法（根据波动率调整采样频率）
- 变化点检测（Z-score 统计检验）
- SQLite WAL 模式存储

## 快速开始

```bash
# 初始化数据库
./target/release/ploy-clean init-db

# 运行 ETL 流程
./target/release/ploy-clean run

# 查看统计
./target/release/ploy-clean stats

# 获取所有市场列表
./target/release/ploy-clean fetch-markets

# 获取单个市场详情
./target/release/ploy-clean get-market --id <市场ID>

# 搜索市场（结果自动保存到数据库）
./target/release/ploy-clean search-markets --query "Trump"
./target/release/ploy-clean search-markets --query "Bitcoin" --limit 10
./target/release/ploy-clean search-markets --query "nuclear" --scan-pages 50  # 深度扫描

# 对单个市场采样
./target/release/ploy-clean sample --market-id <ID> --token-id <TOKEN>
```

## CLI 命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `init-db` | 初始化数据库 | `./ploy-clean init-db` |
| `fetch-markets` | 获取所有市场列表 | `./ploy-clean fetch-markets` |
| `get-market` | 获取单个市场详情 | `./ploy-clean get-market --id 517310` |
| `search-markets` | 搜索市场并保存到数据库 | `./ploy-clean search-markets -q "Trump"` |
| | | `./ploy-clean search-markets -q "nuclear" --scan-pages 50` |
| `sample` | 对指定市场执行自适应采样 | `./ploy-clean sample -m 517310 -t <TOKEN>` |
| `run` | 运行完整 ETL 流程 | `./ploy-clean run -c 16` |
| `stats` | 显示统计信息（可选指定市场） | `./ploy-clean stats` / `./ploy-clean stats -m 517310` |

**全局参数**（所有命令可用）：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-c, --config` | `config/sampling.toml` | 配置文件路径 |
| `-d, --database` | `polymarket.db` | 数据库文件路径 |
| `-l, --log-level` | `info` | 日志级别 |

## 数据库结构

### 表概览

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `markets` | 市场元数据 | id, question, condition_id, clob_token_ids |
| `price_history` | 历史价格数据 | market_id, token_id, timestamp, price |
| `change_points` | 检测到的变化点 | market_id, token_id, timestamp, price_before, price_after |
| `sampling_plans` | 采样计划记录 | market_id, start_timestamp, end_timestamp, interval_seconds |

### markets 表

存储市场基础信息。

```sql
CREATE TABLE markets (
    id TEXT PRIMARY KEY,
    question TEXT NOT NULL,
    condition_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    outcomes TEXT NOT NULL,                              -- JSON 数组，如 ["Yes", "No"]
    volume REAL,
    liquidity REAL,
    active BOOLEAN NOT NULL DEFAULT 1,
    closed BOOLEAN NOT NULL DEFAULT 0,
    end_date TEXT,                                       -- ISO 8601 日期字符串
    category TEXT,
    clob_token_ids TEXT,                                 -- JSON 数组，token ID 列表
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
-- 索引: idx_markets_active (active, closed)
```

**常用查询**：

```sql
-- 获取所有活跃未关闭的市场
SELECT id, question, clob_token_ids
FROM markets
WHERE active = 1 AND closed = 0;

-- 获取特定市场的 token IDs
SELECT clob_token_ids FROM markets WHERE id = '517310';
-- 返回: '["token1", "token2"]' (JSON 字符串)

-- 按交易量排序
SELECT id, question, volume
FROM markets
WHERE active = 1
ORDER BY volume DESC
LIMIT 10;
```

### price_history 表

存储历史价格数据，核心表。

```sql
CREATE TABLE price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,                          -- Unix 时间戳（秒）
    price REAL NOT NULL,                                 -- 概率值 (0-1)
    volume REAL,
    interval TEXT NOT NULL,                              -- '1m', '5m', '1h', '1d'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);
-- 索引: idx_price_market_timestamp (market_id, timestamp DESC)
-- 索引: idx_price_token_timestamp (token_id, timestamp DESC)
-- 索引: idx_price_timestamp (timestamp DESC)
```

**常用查询**：

```sql
-- 获取特定 token 的最新价格
SELECT * FROM price_history
WHERE token_id = 'xxx'
ORDER BY timestamp DESC
LIMIT 10;

-- 获取特定市场的所有价格历史
SELECT ph.*, m.question
FROM price_history ph
JOIN markets m ON ph.market_id = m.id
WHERE m.id = '517310'
ORDER BY ph.timestamp;

-- 获取时间范围内的价格
SELECT timestamp, price
FROM price_history
WHERE token_id = 'xxx'
  AND timestamp BETWEEN 1704067200 AND 1735689600
ORDER BY timestamp;

-- 计算价格统计
SELECT
    COUNT(*) as count,
    MIN(price) as min_price,
    MAX(price) as max_price,
    AVG(price) as avg_price
FROM price_history
WHERE token_id = 'xxx';
```

### change_points 表

存储检测到的价格变化点。

```sql
CREATE TABLE change_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    price_before REAL,
    price_after REAL,
    change_magnitude REAL,                               -- 绝对值
    confidence REAL,                                     -- 0-1
    detection_method TEXT,                               -- 如 'z_score'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);
-- 索引: idx_change_points_market (market_id, timestamp DESC)
-- 索引: idx_change_points_timestamp (timestamp DESC)
```

**常用查询**：

```sql
-- 获取所有变化点
SELECT cp.*, m.question
FROM change_points cp
JOIN markets m ON cp.market_id = m.id
ORDER BY cp.timestamp DESC;

-- 获取特定市场的变化点
SELECT * FROM change_points
WHERE market_id = '517310'
ORDER BY timestamp DESC;

-- 找出最大变化
SELECT * FROM change_points
ORDER BY change_magnitude DESC
LIMIT 10;
```

### sampling_plans 表

记录采样计划（用于自适应采样）。

```sql
CREATE TABLE sampling_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    start_timestamp INTEGER NOT NULL,
    end_timestamp INTEGER NOT NULL,
    interval_seconds INTEGER NOT NULL,
    status TEXT NOT NULL,                                -- 'pending'|'active'|'completed'|'failed'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);
-- 索引: idx_sampling_plans_status (status, start_timestamp)
```

## 数据关系

```
markets (1) ----< (N) price_history
   |
   +----< (N) change_points
   |
   +----< (N) sampling_plans
```

每个市场有多个价格记录、变化点记录和采样计划。

## API 端点

公开 API，无需认证。详细使用约束见「工具使用指南 > API 约束」。

| API | 端点 | 用途 | 关键参数 |
|-----|------|------|----------|
| Gamma API | `https://gamma-api.polymarket.com/markets` | 市场列表 | `limit`, `offset`, `active`, `closed` |
| CLOB API | `https://clob.polymarket.com/prices-history` | 价格历史 | `market`(Token ID), `interval`(`1m`/`5m`/`1h`/`1d`) |

⚠️ Gamma API 的 `query`/`text_query`/`slug_contains` 等搜索参数**均无效**，只能客户端过滤。

## 高级查询示例

```sql
-- 活跃市场 + 最新价格
SELECT m.id, m.question,
    (SELECT price FROM price_history WHERE market_id = m.id ORDER BY timestamp DESC LIMIT 1) as latest_price
FROM markets m WHERE m.active = 1;

-- 滚动平均（最近 100 点）
SELECT timestamp, price,
    AVG(price) OVER (ORDER BY timestamp ROWS BETWEEN 99 PRECEDING AND CURRENT ROW) as moving_avg
FROM price_history WHERE token_id = 'xxx' ORDER BY timestamp;

-- 导出 CSV
.headers on
.mode csv
.output prices.csv
SELECT timestamp, price FROM price_history WHERE token_id = 'xxx';
.output stdout
```

## 配置文件

`config/sampling.toml` 完整配置：

```toml
[sampling]
base_interval_hours = 1
min_interval_minutes = 1
max_interval_hours = 24

# 波动率阈值（决定采样频率）
high_volatility_threshold = 0.02          # 高波动 → 1-5 分钟
medium_volatility_threshold = 0.005       # 中波动 → 15-30 分钟
low_volatility_threshold = 0.001          # 低波动 → 1-6 小时

# 变化率阈值
velocity_threshold_per_hour = 0.01
acceleration_threshold_per_hour_squared = 0.005

# 变化点检测
change_point_detection_window_size = 24
change_point_confidence_level = 0.95
change_point_min_samples = 12

# 采样策略
volatility_multiplier = 10.0
decay_factor = 0.9
stable_period_hours = 2

[api]
gamma_api_base_url = "https://gamma-api.polymarket.com"
clob_api_base_url = "https://clob.polymarket.com"
max_retries = 3
timeout_seconds = 30
rate_limit_per_minute = 60
max_concurrent_requests = 16

[database]
url = "sqlite:///data/polymarket.db"
wal_mode = true
cache_size_mb = 64

[markets]
min_volume = 1000
max_markets = 1000
max_search_results = 50                   # search-markets 最大返回数量
max_scan_pages = 20                       # 扫描页数（推荐 80）
categories = ["politics", "economics", "sports", "crypto"]

[logging]
level = "info"
file = "logs/polymarket.log"
max_file_size_mb = 100
```

## 代码结构

```
src/
├── main.rs           # 入口 + 命令处理逻辑
├── cli.rs            # CLI 参数定义（clap derive）
├── config.rs         # 配置文件解析
├── models/           # 数据模型
│   ├── market.rs     # Market, GammaMarket
│   └── price.rs      # PricePoint, ChangePoint, Interval
├── api/              # API 客户端
│   ├── client.rs     # HTTP 客户端基础
│   ├── gamma.rs      # Gamma API（市场列表/搜索）
│   └── clob.rs       # CLOB API（价格历史）
├── sampling/         # 自适应采样
│   ├── detector.rs   # 变化点检测（Z-score）
│   ├── strategy.rs   # 采样策略（波动率自适应）
│   └── planner.rs    # 采样计划生成
├── database/         # 数据库层
│   ├── schema.rs     # 数据库连接 + 初始化
│   └── models.rs     # CRUD 操作（upsert_market 等）
└── utils/            # 工具函数
    ├── stats.rs      # 统计算法（均值/标准差/Z-score）
    └── time.rs       # 时间处理
```

## 开发注意事项

1. **时间戳**: 所有时间戳为 Unix 秒级时间戳
2. **价格范围**: 价格值在 0-1 之间（概率值）
3. **JSON 字段**: `outcomes` 和 `clob_token_ids` 存储 JSON 字符串，需要解析
4. **WAL 模式**: 数据库使用 WAL 模式，会有 `-wal` 和 `-shm` 临时文件
5. **级联删除**: 所有外键设置 `ON DELETE CASCADE`，删除市场会级联清除关联数据
6. **默认时间戳**: `created_at`/`updated_at` 使用 `DEFAULT (strftime('%s', 'now'))` 自动填充
7. **索引**: 已在 `migrations/schema.sql` 中为所有查询热路径建立索引

> **工具使用指南、典型工作流、研究论文编写流程** 已迁移至 CLI 帮助系统，运行以下命令查看：
> ```bash
> ./target/release/ploy-clean --help              # 总览 + 工作流 + 论文流程
> ./target/release/ploy-clean search-markets --help # 搜索详细指南
> ./target/release/ploy-clean sample --help        # 采样详细指南
> ./target/release/ploy-clean stats --help         # 统计信息
> ```

### 已有论文参考

| 文件 | 主题 | 写作日期 |
|------|------|---------|
| `docs/geopolitical_market_sentiment.md` | 地缘政治市场情绪分析（俄乌/台海/中东/美国/核风险） | 2026-03-01 |

## 故障排查

```bash
# 检查数据库完整性
sqlite3 polymarket.db "PRAGMA integrity_check;"

# 查看表大小
sqlite3 polymarket.db "
SELECT name, (pgsize * 100) as size_kb
FROM pragma_page_count()
JOIN pragma_database_list();
"

# 压缩数据库
sqlite3 polymarket.db "VACUUM;"
```
