// Poly-Cleaner: Polymarket 高性能自适应采样 ETL 工具

mod api;
mod cli;
mod config;
mod database;
mod models;
mod sampling;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use config::Config;
use database::Database;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志
    let log_level = match args.log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    // 加载配置
    let config = Config::load_or_default(&args.config);
    info!("Loaded configuration from {}", args.config.display());

    // 执行命令
    match args.command.unwrap_or(cli::Commands::Run {
        concurrency: config.api.max_concurrent_requests,
        category: None,
    }) {
        cli::Commands::InitDb => {
            init_database(&args.database)?;
        }
        cli::Commands::FetchMarkets { active } => {
            fetch_markets(&config, active).await?;
        }
        cli::Commands::GetMarket { id } => {
            get_market(&config, &id).await?;
        }
        cli::Commands::Sample { market_id, token_id } => {
            sample_market(&config, &args.database, &market_id, &token_id).await?;
        }
        cli::Commands::Run { concurrency, category } => {
            run_etl(&config, &args.database, concurrency, category).await?;
        }
        cli::Commands::Stats { market_id } => {
            show_stats(&args.database, market_id)?;
        }
    }

    Ok(())
}

/// 初始化数据库
fn init_database(db_path: &std::path::PathBuf) -> Result<()> {
    info!("Initializing database at {}", db_path.display());

    let db = Database::open(db_path)?;
    db.init_schema()?;

    info!("Database initialized successfully");
    Ok(())
}

/// 获取市场列表
async fn fetch_markets(config: &Config, _active: bool) -> Result<()> {
    info!("Fetching markets from Gamma API");

    let gamma_client = api::GammaClient::new(&config.api.gamma_api_base_url)?;
    let markets = gamma_client.fetch_markets().await?;

    println!("\n=== Fetched {} Markets ===\n", markets.len());

    for (i, market) in markets.iter().take(20).enumerate() {
        println!("{}. {} ({})", i + 1, market.question, market.id);
        println!("   Active: {}, Volume: {:?}", market.active, market.volume);
    }

    if markets.len() > 20 {
        println!("... and {} more", markets.len() - 20);
    }

    Ok(())
}

/// 获取单个市场详情
async fn get_market(config: &Config, market_id: &str) -> Result<()> {
    info!("Fetching market details for {}", market_id);

    let gamma_client = api::GammaClient::new(&config.api.gamma_api_base_url)?;
    let market = gamma_client.fetch_market(market_id).await?;

    println!("\n=== Market Details ===\n");
    println!("ID: {}", market.id);
    println!("Question: {}", market.question);
    println!("Condition ID: {}", market.condition_id);
    println!("Slug: {}", market.slug);
    println!("Outcomes: {:?}", market.outcomes);
    println!("Volume: {:?}", market.volume);
    println!("Liquidity: {:?}", market.liquidity);
    println!("Active: {}", market.active);
    println!("Closed: {}", market.closed);
    println!("End Date: {:?}", market.end_date);
    println!("Category: {:?}", market.category);
    println!("Token IDs: {:?}", market.clob_token_ids);

    Ok(())
}

/// 对单个市场执行采样
async fn sample_market(
    config: &Config,
    db_path: &std::path::PathBuf,
    market_id: &str,
    token_id: &str,
) -> Result<()> {
    info!("Sampling market {} with token {}", market_id, token_id);

    let _db = Database::open(db_path)?;
    let clob_client = Arc::new(api::ClobClient::new(&config.api.clob_api_base_url)?);

    // 获取粗粒度数据
    info!("Fetching coarse-grained price history...");
    let coarse_data = clob_client
        .fetch_price_history(token_id, models::Interval::OneDay)
        .await?;

    info!("Fetched {} price points", coarse_data.len());

    // 检测变化点
    let detector = sampling::ChangePointDetector::new(
        config.sampling.change_point_detection_window_size,
        config.sampling.change_point_min_samples,
        config.sampling.change_point_confidence_level,
    );

    let change_points = detector.detect_change_points(&coarse_data);
    info!("Detected {} change points", change_points.len());

    for (i, cp) in change_points.iter().enumerate() {
        println!(
            "  {}. {} - Change: {:.3} -> {:.3} ({:.4})",
            i + 1,
            utils::format_timestamp(cp.timestamp),
            cp.price_before,
            cp.price_after,
            cp.change_magnitude
        );
    }

    Ok(())
}

/// 运行完整 ETL 流程
async fn run_etl(
    config: &Config,
    db_path: &std::path::PathBuf,
    concurrency: usize,
    category: Option<String>,
) -> Result<()> {
    info!("Starting ETL process with concurrency {}", concurrency);

    // 初始化数据库
    let db = Database::open(db_path)?;
    db.init_schema()?;

    // 获取市场列表
    info!("Fetching markets from Gamma API...");
    let gamma_client = api::GammaClient::new(&config.api.gamma_api_base_url)?;
    let markets = gamma_client.fetch_markets().await?;

    // 过滤市场
    let markets: Vec<_> = markets
        .into_iter()
        .filter(|m| {
            if m.active && m.clob_token_ids.is_some() {
                if let Some(ref cat) = category {
                    m.category.as_ref().map(|c| c == cat).unwrap_or(false)
                } else {
                    true
                }
            } else {
                false
            }
        })
        .take(config.markets.max_markets)
        .collect();

    info!("Processing {} markets", markets.len());

    // 创建进度条
    let bar = ProgressBar::new(markets.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .expect("invalid template")
            .progress_chars("##-"),
    );

    // 保存市场信息
    for market in &markets {
        if let Err(e) = db.upsert_market(market) {
            warn!("Failed to upert market {}: {}", market.id, e);
        }
    }

    // 并发处理市场
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let mut tasks = Vec::new();

    for market in markets {
        let permit = semaphore.clone().acquire_owned().await?;
        let db_clone = db.clone();
        let clob_client = Arc::new(api::ClobClient::new(&config.api.clob_api_base_url)?);

        let task = tokio::spawn(async move {
            let _permit = permit; // 持有 permit 直到任务完成

            if let Some(token_ids) = &market.clob_token_ids {
                if !token_ids.is_empty() {
                    let token_id = &token_ids[0];
                    match clob_client.fetch_price_history(token_id, models::Interval::OneDay).await {
                        Ok(prices) => {
                            if !prices.is_empty() {
                                let stored_points: Vec<database::StoredPricePoint> = prices
                                    .into_iter()
                                    .map(|p| database::StoredPricePoint {
                                        market_id: market.id.clone(),
                                        token_id: token_id.clone(),
                                        timestamp: p.timestamp,
                                        price: p.price,
                                        volume: p.volume,
                                        interval: "1d".to_string(),
                                    })
                                    .collect();

                                if let Err(e) = db_clone.insert_price_points(&stored_points) {
                                    error!("Failed to insert prices for {}: {}", market.id, e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to fetch prices for {}: {}", market.id, e);
                        }
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        });

        tasks.push(task);
        bar.inc(1);
    }

    // 等待所有任务完成
    for task in tasks {
        let _ = task.await;
    }

    bar.finish_with_message("ETL process completed!");
    info!("ETL process completed successfully");

    Ok(())
}

/// 显示统计信息
fn show_stats(db_path: &std::path::PathBuf, market_id: Option<String>) -> Result<()> {
    let db = Database::open(db_path)?;

    if let Some(market_id) = market_id {
        let stats = db.get_market_stats(&market_id)?;
        println!("\n=== Market Stats: {} ===\n", market_id);
        println!("Price Points: {}", stats.price_count);
        println!("First Timestamp: {}", utils::format_timestamp(stats.first_timestamp));
        println!("Last Timestamp: {}", utils::format_timestamp(stats.last_timestamp));
        println!("Average Price: {:.4}", stats.avg_price);
    } else {
        let markets = db.get_active_markets()?;
        println!("\n=== Database Stats ===\n");
        println!("Total Active Markets: {}", markets.len());

        for market in markets.iter().take(10) {
            println!("  - {} ({})", market.question, market.id);
        }

        if markets.len() > 10 {
            println!("  ... and {} more", markets.len() - 10);
        }
    }

    Ok(())
}
