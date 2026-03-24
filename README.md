# Poly-Cleaner

High-performance Polymarket data cleaning tool based on Rust, implementing **adaptive sampling based on probability change rate**.

## Features

- **Adaptive Sampling Algorithm** - Automatically adjusts sampling frequency based on price volatility
- **Change Point Detection** - Automatically identifies market turning points and performs intensive sampling
- **No Authentication Required** - Uses public Polymarket/Gamma APIs
- **Configuration Driven** - TOML configuration file manages all parameters
- **Incremental Updates** - Resumable data fetching
- **SQLite WAL** - Efficient local storage

## Installation

```bash
# Clone repository
git clone <repository-url>
cd ploy-clean

# Build project
cargo build --release

# Binary located at ./target/release/ploy-clean
```

## Configuration

Edit `config/sampling.toml` to customize sampling behavior:

```toml
[sampling]
# Base sampling interval
base_interval_hours = 1
min_interval_minutes = 1
max_interval_hours = 24

# Volatility thresholds
high_volatility_threshold = 0.02
medium_volatility_threshold = 0.005

# Change point detection
change_point_detection_window_size = 24
change_point_confidence_level = 0.95

[api]
gamma_api_base_url = "https://gamma-api.polymarket.com"
clob_api_base_url = "https://clob.polymarket.com"
max_concurrent_requests = 16

[markets]
# Market search configuration
max_search_results = 50     # Max search results (default 50)
max_scan_pages = 20         # Scan pages (100 per page, default 20=2000 markets)
```

## Usage

```bash
# Initialize database
./target/release/ploy-clean init-db

# Fetch all markets list
./target/release/ploy-clean fetch-markets

# Get single market details
./target/release/ploy-clean get-market --id <MARKET_ID>

# Search markets (results automatically saved to database)
./target/release/ploy-clean search-markets --query "Trump"
./target/release/ploy-clean search-markets --query "Bitcoin" --limit 10
./target/release/ploy-clean search-markets --query "nuclear" --scan-pages 50  # Deep scan

# Perform adaptive sampling on specified market
./target/release/ploy-clean sample --market-id <ID> --token-id <TOKEN>

# Run complete ETL pipeline
./target/release/ploy-clean run

# View statistics
./target/release/ploy-clean stats
```

### Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `init-db` | Initialize database | `./ploy-clean init-db` |
| `fetch-markets` | Fetch all markets list | `./ploy-clean fetch-markets` |
| `get-market` | Get single market details | `./ploy-clean get-market --id 517310` |
| `search-markets` | Search markets and save to database | `./ploy-clean search-markets -q "Trump"` |
| | | `./ploy-clean search-markets -q "nuclear" --scan-pages 50` |
| `sample` | Perform adaptive sampling on specified market | `./ploy-clean sample -m 517310 -t <TOKEN>` |
| `run` | Run complete ETL pipeline | `./ploy-clean run` |
| `stats` | Display statistics | `./ploy-clean stats` |

## Adaptive Sampling Principle

```
┌─────────────────────────────────────────────────────────────────┐
│                        Timeline                                 │
├─────────────────────────────────────────────────────────────────┤
│  Stable    │  Turning    │  High      │    Stable     │ Current │
│  Period    │  Point      │  Volatility│    Period     │         │
│  (sparse)  │  (dense)    │  (densest) │    (sparse)   │ (dense) │
│  1d interval │ 1h interval│ 1m-5m      │   1h interval│ 1m      │
└─────────────────────────────────────────────────────────────────┘
```

### Change Point Detection Algorithm

- **First Derivative** (Velocity): `ΔP/Δt` - Price change rate
- **Second Derivative** (Acceleration): `Δ²P/Δt²` - Rate of change acceleration
- **Volatility**: Rolling window standard deviation - Market stability
- **Statistical Test**: Z-score or CUSUM to detect significant change points

### Sampling Levels

| Volatility | Sampling Interval | Level |
|------------|-------------------|-------|
| > 2% | 1-5 minutes | High |
| 0.5% - 2% | 15-30 minutes | Medium |
| < 0.5% | 1-6 hours | Low |

## Project Structure

```
ploy-clean/
├── Cargo.toml
├── config/
│   └── sampling.toml
├── migrations/
│   └── schema.sql
└── src/
    ├── main.rs           # Program entry
    ├── cli.rs            # CLI definition
    ├── config.rs         # Configuration loading
    ├── models/           # Data models
    ├── api/              # API clients
    ├── sampling/         # Adaptive sampling algorithms
    ├── database/         # Database layer
    └── utils/            # Utility functions
```

## Database

Uses SQLite with the following tables:

- `markets` - Market information
- `price_history` - Historical price data
- `change_points` - Detected change points
- `sampling_plans` - Sampling plan records

## Development

```bash
# Run tests
cargo test

# Build
cargo build --release

# Run
./target/release/ploy-clean --help
```

## Dependencies

- tokio - Async runtime
- reqwest - HTTP client
- rusqlite - SQLite bindings
- serde - Serialization/deserialization
- anyhow - Error handling
- tracing - Logging

## License

MIT
