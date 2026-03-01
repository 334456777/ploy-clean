// CLI 参数解析

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Poly-Cleaner: Polymarket 高性能自适应采样 ETL 工具
#[derive(Parser, Debug)]
#[command(name = "ploy-clean")]
#[command(author = "yusteven")]
#[command(version = "0.1.0")]
#[command(about = "High-performance adaptive sampling ETL tool for Polymarket data", long_about = None)]
pub struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config/sampling.toml")]
    pub config: PathBuf,

    /// 数据库文件路径
    #[arg(short, long, default_value = "polymarket.db")]
    pub database: PathBuf,

    /// 日志级别
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 获取所有市场列表
    FetchMarkets {
        /// 只获取活跃市场
        #[arg(short, long, default_value_t = true)]
        active: bool,
    },

    /// 获取单个市场详情
    GetMarket {
        /// 市场 ID
        #[arg(short, long)]
        id: String,
    },

    /// 对指定市场执行自适应采样
    Sample {
        /// 市场 ID
        #[arg(short, long)]
        market_id: String,

        /// Token ID
        #[arg(short, long)]
        token_id: String,
    },

    /// 运行完整的 ETL 流程
    Run {
        /// 最大并发市场数
        #[arg(short, long, default_value_t = 16)]
        concurrency: usize,

        /// 只处理指定分类
        #[arg(short, long)]
        category: Option<String>,
    },

    /// 初始化数据库
    InitDb,

    /// 搜索市场
    SearchMarkets {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 最大返回数量（覆盖配置文件中的默认值）
        #[arg(short, long)]
        limit: Option<usize>,

        /// 扫描页数（每页 100 个市场，默认 20 页=2000 市场，--scan-pages 50 可扫描 5000 个）
        #[arg(long)]
        scan_pages: Option<usize>,
    },

    /// 显示统计信息
    Stats {
        /// 市场 ID
        #[arg(short, long)]
        market_id: Option<String>,
    },
}
