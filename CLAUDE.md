# Poly-Cleaner - AI Development Guide

## Project Overview

Poly-Cleaner is a high-performance Polymarket data ETL tool based on Rust, implementing adaptive sampling based on probability change rate.

**Core Features**:
- Fetch prediction market data from Polymarket/Gamma API
- Adaptive sampling algorithm (adjusts sampling frequency based on volatility)
- Change point detection (Z-score statistical test)
- SQLite WAL mode storage

## Quick Start

```bash
# Initialize database
./target/release/ploy-clean init-db

# Run ETL pipeline
./target/release/ploy-clean run

# View statistics
./target/release/ploy-clean stats

# Fetch all markets list
./target/release/ploy-clean fetch-markets

# Get single market details
./target/release/ploy-clean get-market --id <MARKET_ID>

# Search markets (results automatically saved to database)
./target/release/ploy-clean search-markets --query "Trump"
./target/release/ploy-clean search-markets --query "Bitcoin" --limit 10
./target/release/ploy-clean search-markets --query "nuclear" --scan-pages 50  # Deep scan

# Sample single market
./target/release/ploy-clean sample --market-id <ID> --token-id <TOKEN>
```

## CLI Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init-db` | Initialize database | `./ploy-clean init-db` |
| `fetch-markets` | Fetch all markets list | `./ploy-clean fetch-markets` |
| `get-market` | Get single market details | `./ploy-clean get-market --id 517310` |
| `search-markets` | Search markets and save to database | `./ploy-clean search-markets -q "Trump"` |
| | | `./ploy-clean search-markets -q "nuclear" --scan-pages 50` |
| `sample` | Perform adaptive sampling on specified market | `./ploy-clean sample -m 517310 -t <TOKEN>` |
| `run` | Run complete ETL pipeline | `./ploy-clean run -c 16` |
| `stats` | Display statistics (optionally specify market) | `./ploy-clean stats` / `./ploy-clean stats -m 517310` |

**Global Parameters** (available for all commands):

| Parameter | Default | Description |
|-----------|---------|-------------|
| `-c, --config` | `config/sampling.toml` | Configuration file path |
| `-d, --database` | `polymarket.db` | Database file path |
| `-l, --log-level` | `info` | Log level |

## Database Structure

### Table Overview

| Table Name | Purpose | Key Fields |
|------------|---------|------------|
| `markets` | Market metadata | id, question, condition_id, clob_token_ids |
| `price_history` | Historical price data | market_id, token_id, timestamp, price |
| `change_points` | Detected change points | market_id, token_id, timestamp, price_before, price_after |
| `sampling_plans` | Sampling plan records | market_id, start_timestamp, end_timestamp, interval_seconds |

### markets Table

Stores market basic information.

```sql
CREATE TABLE markets (
    id TEXT PRIMARY KEY,
    question TEXT NOT NULL,
    condition_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    outcomes TEXT NOT NULL,                              -- JSON array, e.g. ["Yes", "No"]
    volume REAL,
    liquidity REAL,
    active BOOLEAN NOT NULL DEFAULT 1,
    closed BOOLEAN NOT NULL DEFAULT 0,
    end_date TEXT,                                       -- ISO 8601 date string
    category TEXT,
    clob_token_ids TEXT,                                 -- JSON array, token ID list
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
-- Index: idx_markets_active (active, closed)
```

**Common Queries**:

```sql
-- Get all active non-closed markets
SELECT id, question, clob_token_ids
FROM markets
WHERE active = 1 AND closed = 0;

-- Get specific market's token IDs
SELECT clob_token_ids FROM markets WHERE id = '517310';
-- Returns: '["token1", "token2"]' (JSON string)

-- Sort by volume
SELECT id, question, volume
FROM markets
WHERE active = 1
ORDER BY volume DESC
LIMIT 10;
```

### price_history Table

Stores historical price data, core table.

```sql
CREATE TABLE price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,                          -- Unix timestamp (seconds)
    price REAL NOT NULL,                                 -- Probability value (0-1)
    volume REAL,
    interval TEXT NOT NULL,                              -- '1m', '5m', '1h', '1d'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);
-- Index: idx_price_market_timestamp (market_id, timestamp DESC)
-- Index: idx_price_token_timestamp (token_id, timestamp DESC)
-- Index: idx_price_timestamp (timestamp DESC)
```

**Common Queries**:

```sql
-- Get latest prices for specific token
SELECT * FROM price_history
WHERE token_id = 'xxx'
ORDER BY timestamp DESC
LIMIT 10;

-- Get all price history for specific market
SELECT ph.*, m.question
FROM price_history ph
JOIN markets m ON ph.market_id = m.id
WHERE m.id = '517310'
ORDER BY ph.timestamp;

-- Get prices within time range
SELECT timestamp, price
FROM price_history
WHERE token_id = 'xxx'
  AND timestamp BETWEEN 1704067200 AND 1735689600
ORDER BY timestamp;

-- Calculate price statistics
SELECT
    COUNT(*) as count,
    MIN(price) as min_price,
    MAX(price) as max_price,
    AVG(price) as avg_price
FROM price_history
WHERE token_id = 'xxx';
```

### change_points Table

Stores detected price change points.

```sql
CREATE TABLE change_points (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id TEXT NOT NULL,
    token_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    price_before REAL,
    price_after REAL,
    change_magnitude REAL,                               -- Absolute value
    confidence REAL,                                     -- 0-1
    detection_method TEXT,                               -- e.g. 'z_score'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(market_id) REFERENCES markets(id) ON DELETE CASCADE
);
-- Index: idx_change_points_market (market_id, timestamp DESC)
-- Index: idx_change_points_timestamp (timestamp DESC)
```

**Common Queries**:

```sql
-- Get all change points
SELECT cp.*, m.question
FROM change_points cp
JOIN markets m ON cp.market_id = m.id
ORDER BY cp.timestamp DESC;

-- Get change points for specific market
SELECT * FROM change_points
WHERE market_id = '517310'
ORDER BY timestamp DESC;

-- Find maximum changes
SELECT * FROM change_points
ORDER BY change_magnitude DESC
LIMIT 10;
```

### sampling_plans Table

Records sampling plans (for adaptive sampling).

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
-- Index: idx_sampling_plans_status (status, start_timestamp)
```

## Data Relationships

```
markets (1) ----< (N) price_history
   |
   +----< (N) change_points
   |
   +----< (N) sampling_plans
```

Each market has multiple price records, change point records, and sampling plans.

## API Endpoints

Public APIs, no authentication required. For detailed usage constraints, see "Tool Usage Guide > API Constraints".

| API | Endpoint | Purpose | Key Parameters |
|-----|----------|---------|----------------|
| Gamma API | `https://gamma-api.polymarket.com/markets` | Markets list | `limit`, `offset`, `active`, `closed` |
| CLOB API | `https://clob.polymarket.com/prices-history` | Price history | `market`(Token ID), `interval`(`1m`/`5m`/`1h`/`1d`) |

⚠️ Gamma API's `query`/`text_query`/`slug_contains` and other search parameters **are all invalid**, only client-side filtering is possible.

## Advanced Query Examples

```sql
-- Active markets + latest price
SELECT m.id, m.question,
    (SELECT price FROM price_history WHERE market_id = m.id ORDER BY timestamp DESC LIMIT 1) as latest_price
FROM markets m WHERE m.active = 1;

-- Rolling average (last 100 points)
SELECT timestamp, price,
    AVG(price) OVER (ORDER BY timestamp ROWS BETWEEN 99 PRECEDING AND CURRENT ROW) as moving_avg
FROM price_history WHERE token_id = 'xxx' ORDER BY timestamp;

-- Export CSV
.headers on
.mode csv
.output prices.csv
SELECT timestamp, price FROM price_history WHERE token_id = 'xxx';
.output stdout
```

## Configuration File

Complete `config/sampling.toml` configuration:

```toml
[sampling]
base_interval_hours = 1
min_interval_minutes = 1
max_interval_hours = 24

# Volatility thresholds (determine sampling frequency)
high_volatility_threshold = 0.02          # High volatility → 1-5 min
medium_volatility_threshold = 0.005       # Medium volatility → 15-30 min
low_volatility_threshold = 0.001          # Low volatility → 1-6 h

# Rate of change thresholds
velocity_threshold_per_hour = 0.01
acceleration_threshold_per_hour_squared = 0.005

# Change point detection
change_point_detection_window_size = 24
change_point_confidence_level = 0.95
change_point_min_samples = 12

# Sampling strategy
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
max_search_results = 50                   # Max returns for search-markets
max_scan_pages = 20                       # Scan pages (recommended 80)
categories = ["politics", "economics", "sports", "crypto"]

[logging]
level = "info"
file = "logs/polymarket.log"
max_file_size_mb = 100
```

## Code Structure

```
src/
├── main.rs           # Entry + command handling logic
├── cli.rs            # CLI parameter definition (clap derive)
├── config.rs         # Configuration file parsing
├── models/           # Data models
│   ├── market.rs     # Market, GammaMarket
│   └── price.rs      # PricePoint, ChangePoint, Interval
├── api/              # API clients
│   ├── client.rs     # HTTP client base
│   ├── gamma.rs      # Gamma API (markets list/search)
│   └── clob.rs       # CLOB API (price history)
├── sampling/         # Adaptive sampling
│   ├── detector.rs   # Change point detection (Z-score)
│   ├── strategy.rs   # Sampling strategy (volatility adaptive)
│   └── planner.rs    # Sampling plan generation
├── database/         # Database layer
│   ├── schema.rs     # Database connection + initialization
│   └── models.rs     # CRUD operations (upsert_market etc.)
└── utils/            # Utility functions
    ├── stats.rs      # Statistical algorithms (mean/std dev/Z-score)
    └── time.rs       # Time handling
```

## Development Notes

1. **Timestamps**: All timestamps are Unix second-level timestamps
2. **Price Range**: Price values are between 0-1 (probability values)
3. **JSON Fields**: `outcomes` and `clob_token_ids` store JSON strings, need parsing
4. **WAL Mode**: Database uses WAL mode, will have `-wal` and `-shm` temporary files
5. **Cascade Delete**: All foreign keys set `ON DELETE CASCADE`, deleting a market will cascade clear associated data
6. **Default Timestamps**: `created_at`/`updated_at` use `DEFAULT (strftime('%s', 'now'))` for auto-fill
7. **Indexes**: Already established in `migrations/schema.sql` for all query hot paths

> **Tool usage guide, typical workflows, research paper writing process** have been migrated to CLI help system, run the following commands to view:
> ```bash
> ./target/release/ploy-clean --help              # Overview + workflow + paper process
> ./target/release/ploy-clean search-markets --help # Search detailed guide
> ./target/release/ploy-clean sample --help        # Sampling detailed guide
> ./target/release/ploy-clean stats --help         # Statistics
> ```

### Existing Paper References

| File | Topic | Writing Date |
|------|-------|--------------|
| `docs/geopolitical_market_sentiment.md` | Geopolitical market sentiment analysis (Russia-Ukraine/Taiwan Strait/Middle East/US/Nuclear risk) | 2026-03-01 |

## Troubleshooting

```bash
# Check database integrity
sqlite3 polymarket.db "PRAGMA integrity_check;"

# View table sizes
sqlite3 polymarket.db "
SELECT name, (pgsize * 100) as size_kb
FROM pragma_page_count()
JOIN pragma_database_list();
"

# Compact database
sqlite3 polymarket.db "VACUUM;"
```
