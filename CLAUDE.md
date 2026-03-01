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
| `run` | 运行完整 ETL 流程 | `./ploy-clean run` |
| `stats` | 显示统计信息 | `./ploy-clean stats` |

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
    id TEXT PRIMARY KEY,              -- 市场 ID
    question TEXT NOT NULL,           -- 问题描述
    condition_id TEXT NOT NULL,       -- 条件 ID
    slug TEXT NOT NULL,               -- URL slug
    outcomes TEXT NOT NULL,           -- JSON 数组，如 ["Yes", "No"]
    volume REAL,                      -- 交易量
    liquidity REAL,                   -- 流动性
    active BOOLEAN NOT NULL,          -- 是否活跃
    closed BOOLEAN NOT NULL,          -- 是否已关闭
    end_date TEXT,                    -- ISO 8601 结束日期
    category TEXT,                    -- 分类
    clob_token_ids TEXT,              -- JSON 数组，token ID 列表
    created_at INTEGER,               -- 创建时间（Unix 时间戳）
    updated_at INTEGER                -- 更新时间（Unix 时间戳）
);
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
    market_id TEXT NOT NULL,          -- 关联 markets.id
    token_id TEXT NOT NULL,           -- Token ID（用于查询价格）
    timestamp INTEGER NOT NULL,       -- Unix 时间戳（秒）
    price REAL NOT NULL,              -- 价格/概率值 (0-1)
    volume REAL,                      -- 交易量（可选）
    interval TEXT NOT NULL,           -- 采样间隔 ('1m', '5m', '1h', '1d')
    created_at INTEGER,
    FOREIGN KEY(market_id) REFERENCES markets(id)
);
```

**索引**：
- `idx_price_market_timestamp`: (market_id, timestamp DESC)
- `idx_price_token_timestamp`: (token_id, timestamp DESC)
- `idx_price_timestamp`: (timestamp DESC)

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
    timestamp INTEGER NOT NULL,       -- 变化发生时间
    price_before REAL,                -- 变化前价格
    price_after REAL,                 -- 变化后价格
    change_magnitude REAL,            -- 变化幅度（绝对值）
    confidence REAL,                  -- 置信度 (0-1)
    detection_method TEXT,            -- 检测方法（如 'z_score'）
    created_at INTEGER,
    FOREIGN KEY(market_id) REFERENCES markets(id)
);
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
    interval_seconds INTEGER NOT NULL, -- 采样间隔（秒）
    status TEXT NOT NULL,              -- 'pending', 'active', 'completed', 'failed'
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY(market_id) REFERENCES markets(id)
);
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

程序使用以下公开 API（无需认证）：

| API | 端点 | 用途 |
|-----|------|------|
| Gamma API | `https://gamma-api.polymarket.com/markets` | 获取市场列表 |
| CLOB API | `https://clob.polymarket.com/prices-history` | 获取价格历史 |

**Gamma API 参数**：
- `limit`: 返回数量
- `offset`: 偏移量（分页）
- `active=true`: 只获取活跃市场
- `closed=false`: 排除已关闭市场

**CLOB API 参数**：
- `market`: Token ID
- `interval`: 时间间隔 (`1m`, `5m`, `1h`, `1d`)

## 查询示例

### 获取最新市场状态

```sql
-- 查看活跃市场数量和最新价格
SELECT
    m.id,
    m.question,
    m.active,
    m.closed,
    (SELECT price FROM price_history WHERE market_id = m.id ORDER BY timestamp DESC LIMIT 1) as latest_price
FROM markets m
WHERE m.active = 1;
```

### 价格趋势分析

```sql
-- 计算滚动平均价格（最近 100 个点）
SELECT
    timestamp,
    price,
    AVG(price) OVER (
        ORDER BY timestamp
        ROWS BETWEEN 99 PRECEDING AND CURRENT ROW
    ) as moving_avg
FROM price_history
WHERE token_id = 'xxx'
ORDER BY timestamp;
```

### 变化点分析

```sql
-- 获取变化点前后的价格数据
SELECT
    cp.timestamp as change_time,
    cp.price_before,
    cp.price_after,
    cp.change_magnitude,
    (SELECT price FROM price_history ph
     WHERE ph.token_id = cp.token_id
       AND ph.timestamp < cp.timestamp
     ORDER BY ph.timestamp DESC LIMIT 10) as prices_before
FROM change_points cp
WHERE cp.market_id = '517310'
ORDER BY cp.timestamp DESC;
```

## 数据导出

```sql
-- 导出为 CSV
.headers on
.mode csv
.output prices.csv
SELECT timestamp, price FROM price_history WHERE token_id = 'xxx';
.output stdout

-- 导出为 JSON（需要 sqlite3 扩展）
-- 或使用程序读取数据后导出
```

## 配置文件

`config/sampling.toml` 控制采样行为：

```toml
[sampling]
base_interval_hours = 1      # 基础采样间隔
min_interval_minutes = 1     # 最小间隔
max_interval_hours = 24      # 最大间隔

# 波动率阈值
high_volatility_threshold = 0.02     # 高波动: 1-5 分钟
medium_volatility_threshold = 0.005  # 中波动: 15-30 分钟
low_volatility_threshold = 0.001     # 低波动: 1-6 小时

# 变化点检测
change_point_detection_window_size = 24  # 滑动窗口大小
change_point_confidence_level = 0.95     # 置信度

[markets]
# 市场搜索配置
max_search_results = 50                  # 搜索最大返回数量（默认 50）
max_scan_pages = 20                      # 搜索扫描页数（每页 100，默认 20=2000 市场）
```

## 代码结构

```
src/
├── main.rs           # CLI 入口
├── models/           # 数据模型
│   ├── market.rs     # Market, GammaMarket
│   └── price.rs      # PricePoint, ChangePoint, Interval
├── api/              # API 客户端
│   ├── gamma.rs      # Gamma API
│   └── clob.rs       # CLOB API
├── sampling/         # 自适应采样
│   ├── detector.rs   # 变化点检测
│   ├── strategy.rs   # 采样策略
│   └── planner.rs    # 采样计划
├── database/         # 数据库层
│   ├── schema.rs     # 数据库连接
│   └── models.rs     # ORM 操作
└── utils/            # 工具函数
    ├── stats.rs      # 统计算法
    └── time.rs       # 时间处理
```

## 开发注意事项

1. **时间戳**: 所有时间戳为 Unix 秒级时间戳
2. **价格范围**: 价格值在 0-1 之间（概率值）
3. **JSON 字段**: `outcomes` 和 `clob_token_ids` 存储 JSON 字符串，需要解析
4. **WAL 模式**: 数据库使用 WAL 模式，会有 `-wal` 和 `-shm` 临时文件
5. **索引**: 重要查询已建索引，充分利用可提高性能

## 工具使用指南

### search-markets：市场搜索

**核心机制**：Gamma API **不支持**服务端关键词搜索。`search-markets` 实际工作方式是批量拉取市场列表后在客户端做 AND 关键词匹配。搜索结果自动 upsert 到 `markets` 表。

**基本用法**：
```bash
./ploy-clean search-markets --query "<关键词>" --limit <返回上限> --scan-pages <扫描页数>
```

**参数选择**：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--query` | 必填 | 空格分隔的关键词，AND 匹配，**必须用英文** |
| `--limit` | 50 | 最大返回条数。精准关键词用 20，模糊关键词用 50 |
| `--scan-pages` | 20 | 每页 100 个市场。**推荐 80**（覆盖绝大多数活跃市场） |

**扫描量参考**：

| scan-pages | 扫描量 | 耗时 | 适用场景 |
|-----------|--------|------|---------|
| 20（默认） | 2000 | ~5s | 快速验证 |
| 50 | 5000 | ~10s | 一般研究 |
| **80** | **8000** | **~18s** | **推荐值**，80 页后通常已无更多市场 |
| 100 | 10000 | ~25s | 极稀有关键词 |

也可在 `config/sampling.toml` 中永久修改：
```toml
[markets]
max_scan_pages = 80
```

**关键词设计原则**：
- **双词组合精准度更高**：`"Ukraine ceasefire"` 比 `"Ukraine"` 噪声少得多
- **人名/专有名词最稳定**：`"Putin"`, `"Bitcoin"` 等直接锚定相关市场
- **单词适合专属领域**：`"Iran"`, `"recession"` 等唯一性强的词
- **宽关键词 + 高 limit**：不确定时用宽关键词 + `--limit 50` 避免遗漏
- **多组关键词覆盖同一主题**：从不同角度（事件名/人名/地名/概念）搜索同一主题，数据库自动去重

### sample：价格采样

**核心机制**：通过 CLOB API 获取某个 token 的实时价格历史（48h 内，1分钟间隔），同时自动运行 Z-score 变化点检测。结果分别写入 `price_history` 和 `change_points` 表。

**基本用法**：
```bash
./ploy-clean sample --market-id <市场ID> --token-id <Token ID>
```

**Token ID 获取方式**：
- `markets` 表的 `clob_token_ids` 字段是 JSON 数组，如 `["tokenA", "tokenB"]`
- 第一个 token 通常对应 **"Yes"** outcome，第二个对应 **"No"**
- 可用 SQL 查询：`SELECT clob_token_ids FROM markets WHERE id = '<市场ID>'`

**注意事项**：
- `search-markets` 返回的 `outcomePrices` 是 Gamma 快照，**可能与 CLOB 实时价格有 1-3% 偏差**
- 需要精确数据时必须用 `sample`
- **部分 token 无 CLOB 数据**（返回 0 条记录），此时换 `clob_token_ids` 数组中的另一个 token
- 每次返回约 50-200 个价格点 + 10-40 个变化点

### stats：查看统计

```bash
./ploy-clean stats
```

显示数据库中的市场总数、价格记录数、变化点数等汇总信息。每次大规模搜索或采样后运行，用于验证数据完整性。

### 典型工作流

```
1. 确定研究主题 → 设计关键词集合（20-30 组，覆盖人/地/事/概念）
2. 批量搜索 → 每个关键词执行 search-markets --scan-pages 80
3. 验证入库 → stats 查看总量 + SQL 查询检查覆盖度
4. 精确采样 → 对核心市场（高交易量/高相关度）执行 sample
5. 数据分析 → 从 markets/price_history/change_points 表提取分析数据
```

### 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| search 返回 0 结果 | 关键词拼写错误或过于精确 | 放宽关键词，如 `"nuclear"` 替代 `"nuclear detonation"` |
| sample 返回 0 价格点 | token ID 无 CLOB 数据 | 换 `clob_token_ids` 数组中的另一个 token |
| 概率值差异大 | Gamma 快照 vs CLOB 实时 | 以 `sample` 的 CLOB 数据为准 |
| 同一市场重复 | 多个关键词命中同一市场 | 正常行为，upsert 自动去重 |
| 搜索无特定市场 | 该市场不在前 N 页 | 增大 `--scan-pages` 或换关键词 |

### API 约束

- **Gamma API 无服务端搜索**：`query`, `text_query`, `slug_contains` 等参数均被忽略，只能客户端过滤
- **CLOB API 偶尔无数据**：部分 token ID 返回空历史，需换另一个 token
- **Gamma 价格快照不精确**：与 CLOB 实时价格有 1-3% 偏差

## 研究论文编写流程

当需要编写基于 Polymarket 数据的研究论文时，按以下流程执行。

### 第一阶段：数据采集

1. **设计关键词集合**（20-30 组）
   - 按维度分类：事件名、地名、人名、组织、概念
   - 每个维度用多种表述覆盖（如同时搜 `"ceasefire"` 和 `"peace deal"`）
   - 所有关键词使用**英文**

2. **批量搜索入库**
   ```bash
   # 推荐参数
   ./ploy-clean search-markets --query "<关键词>" --limit 50 --scan-pages 80
   ```

3. **验证数据完整性**
   ```bash
   ./ploy-clean stats
   ```
   ```sql
   SELECT id, question, volume FROM markets WHERE question LIKE '%关键词%' ORDER BY volume DESC;
   ```

4. **对核心市场采样**（选交易量 Top 和高相关度的市场）
   ```bash
   ./ploy-clean sample --market-id <ID> --token-id <Token>
   ```

### 第二阶段：数据分析

5. **分维度整理**：按研究主题将市场分组，每组记录 ID/问题/概率/交易量/流动性
6. **提取关键指标**：
   - **概率值**：outcome price = 隐含概率
   - **交易量**：信号可靠性（$1M+ 高可靠、$100K-$1M 中等、<$100K 低可靠）
   - **时间梯度**：同一议题不同到期日的概率对比（短期/中期/长期）
7. **情绪分类**：

   | 概率范围 | 情绪标签 |
   |---------|---------|
   | 0-5% | 极度悲观（几乎不可能） |
   | 5-20% | 悲观（不太可能） |
   | 20-40% | 偏悲观（有可能但不太乐观） |
   | 40-60% | 中性/高度不确定 |
   | 60-80% | 偏乐观（较可能） |
   | 80-95% | 乐观（很可能） |
   | 95-100% | 极度乐观（几乎确定） |

8. **跨市场关联**：识别关联结构、时间梯度模式、流动性对信号的影响

### 第三阶段：论文撰写

9. **论文结构**：
   ```
   # 标题（中英双语）
   ## 摘要（300字，核心发现 3-5 点）
   ## 1. 引言（背景、问题、文献）
   ## 2. 方法与数据（工具、分析方法、数据质量说明）
   ## 3. 全景分析（市场分布总览、交易量分布 ASCII 柱状图）
   ## 4. 核心议题深度分析（每维度一节：数据矩阵表格 + 情绪分析 + 结构洞察）
   ## 5. 跨市场传导机制（关联图、时间梯度、流动性矩阵）
   ## 6. 预测市场优势与局限
   ## 7. 结论（核心发现、政策启示、未来方向）
   ## 附录：完整数据表
   ## 参考文献
   ```

10. **数据呈现要求**：
    - 每个议题必须有**数据矩阵表格**（市场/概率/交易量/信号强度）
    - 交易量分布用 ASCII **柱状图**
    - 跨市场关联用 ASCII **流程图/树形图**
    - 概率精确到小数点后一位，交易量用 `$xxM` / `$xxK` 格式
    - 低流动性市场标注 ⚠️ 信号可靠性警告

11. **质量检查清单**：
    - [ ] 所有概率值来自实际 API 数据？
    - [ ] 注明数据快照时间？
    - [ ] 低流动性市场标注信号可靠性？
    - [ ] 时间维度概率梯度分析？
    - [ ] 跨市场关联分析？
    - [ ] 预测市场方法论局限性讨论？

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
