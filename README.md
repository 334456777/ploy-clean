# Poly-Cleaner

基于 Rust 开发的高性能 Polymarket 数据清洗工具，实现**基于概率变化程度的自适应采样**。

## 特性

- **自适应采样算法** - 根据价格波动率自动调整采样频率
- **变化点检测** - 自动识别市场转折点并密集采样
- **无需认证** - 使用公开的 Polymarket/Gamma API
- **配置驱动** - TOML 配置文件管理所有参数
- **断点续传** - 增量数据获取
- **SQLite WAL** - 高效本地存储

## 安装

```bash
# 克隆仓库
git clone <repository-url>
cd ploy-clean

# 构建项目
cargo build --release

# 二进制文件位于 ./target/release/ploy-clean
```

## 配置

编辑 `config/sampling.toml` 来自定义采样行为：

```toml
[sampling]
# 基础采样间隔
base_interval_hours = 1
min_interval_minutes = 1
max_interval_hours = 24

# 波动率阈值
high_volatility_threshold = 0.02
medium_volatility_threshold = 0.005

# 变化点检测
change_point_detection_window_size = 24
change_point_confidence_level = 0.95

[api]
gamma_api_base_url = "https://gamma-api.polymarket.com"
clob_api_base_url = "https://clob.polymarket.com"
max_concurrent_requests = 16

[markets]
# 市场搜索配置
max_search_results = 50     # 搜索最大返回数量（默认 50）
```

## 使用

```bash
# 初始化数据库
./target/release/ploy-clean init-db

# 获取所有市场列表
./target/release/ploy-clean fetch-markets

# 获取单个市场详情
./target/release/ploy-clean get-market --id <市场ID>

# 搜索市场（结果自动保存到数据库）
./target/release/ploy-clean search-markets --query "Trump"
./target/release/ploy-clean search-markets --query "Bitcoin" --limit 10

# 对指定市场执行自适应采样
./target/release/ploy-clean sample --market-id <ID> --token-id <TOKEN>

# 运行完整 ETL 流程
./target/release/ploy-clean run

# 查看统计信息
./target/release/ploy-clean stats
```

### 命令说明

| 命令 | 说明 | 示例 |
|------|------|------|
| `init-db` | 初始化数据库 | `./ploy-clean init-db` |
| `fetch-markets` | 获取所有市场列表 | `./ploy-clean fetch-markets` |
| `get-market` | 获取单个市场详情 | `./ploy-clean get-market --id 517310` |
| `search-markets` | 搜索市场并保存到数据库 | `./ploy-clean search-markets -q "Trump"` |
| `sample` | 对指定市场执行自适应采样 | `./ploy-clean sample -m 517310 -t <TOKEN>` |
| `run` | 运行完整 ETL 流程 | `./ploy-clean run` |
| `stats` | 显示统计信息 | `./ploy-clean stats` |

## 自适应采样原理

```
┌─────────────────────────────────────────────────────────────────┐
│                        时间线                                    │
├─────────────────────────────────────────────────────────────────┤
│  稳定期    │  转折检测   │  高波动期   │    稳定期    │   当前   │
│  (稀疏)    │  (密集)     │  (最密集)  │    (稀疏)    │  (密集)  │
│  1d间隔    │  1h间隔     │  1m-5m间隔 │   1h间隔     │  1m间隔  │
└─────────────────────────────────────────────────────────────────┘
```

### 变化点检测算法

- **一阶变化率** (Velocity): `ΔP/Δt` - 价格变化速度
- **二阶变化率** (Acceleration): `Δ²P/Δt²` - 变化率的变化
- **波动率** (Volatility): 滚动窗口标准差 - 市场稳定性
- **统计检验**: Z-score 或 CUSUM 检测显著变化点

### 采样级别

| 波动率 | 采样间隔 | 级别 |
|--------|----------|------|
| > 2% | 1-5 分钟 | High |
| 0.5% - 2% | 15-30 分钟 | Medium |
| < 0.5% | 1-6 小时 | Low |

## 项目结构

```
ploy-clean/
├── Cargo.toml
├── config/
│   └── sampling.toml
├── migrations/
│   └── schema.sql
└── src/
    ├── main.rs           # 程序入口
    ├── cli.rs            # CLI 定义
    ├── config.rs         # 配置加载
    ├── models/           # 数据模型
    ├── api/              # API 客户端
    ├── sampling/         # 自适应采样算法
    ├── database/         # 数据库层
    └── utils/            # 工具函数
```

## 数据库

使用 SQLite 存储，包含以下表：

- `markets` - 市场信息
- `price_history` - 历史价格数据
- `change_points` - 检测到的变化点
- `sampling_plans` - 采样计划记录

## 开发

```bash
# 运行测试
cargo test

# 构建
cargo build --release

# 运行
./target/release/ploy-clean --help
```

## 依赖

- tokio - 异步运行时
- reqwest - HTTP 客户端
- rusqlite - SQLite 绑定
- serde - 序列化/反序列化
- anyhow - 错误处理
- tracing - 日志

## 许可证

MIT
