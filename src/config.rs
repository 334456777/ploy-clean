// 配置管理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sampling: SamplingConfig,
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub markets: MarketsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub base_interval_hours: f64,
    pub min_interval_minutes: f64,
    pub max_interval_hours: f64,
    pub high_volatility_threshold: f64,
    pub medium_volatility_threshold: f64,
    pub low_volatility_threshold: f64,
    pub velocity_threshold_per_hour: f64,
    pub acceleration_threshold_per_hour_squared: f64,
    pub change_point_detection_window_size: usize,
    pub change_point_confidence_level: f64,
    pub change_point_min_samples: usize,
    pub volatility_multiplier: f64,
    pub decay_factor: f64,
    pub stable_period_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub gamma_api_base_url: String,
    pub clob_api_base_url: String,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
    pub max_concurrent_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub wal_mode: bool,
    pub cache_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsConfig {
    pub min_volume: f64,
    pub max_markets: usize,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub max_file_size_mb: usize,
}

impl Config {
    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// 加载配置，如果文件不存在则返回默认配置
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path.as_ref()).unwrap_or_else(|_| Self::default())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sampling: SamplingConfig {
                base_interval_hours: 1.0,
                min_interval_minutes: 1.0,
                max_interval_hours: 24.0,
                high_volatility_threshold: 0.02,
                medium_volatility_threshold: 0.005,
                low_volatility_threshold: 0.001,
                velocity_threshold_per_hour: 0.01,
                acceleration_threshold_per_hour_squared: 0.005,
                change_point_detection_window_size: 24,
                change_point_confidence_level: 0.95,
                change_point_min_samples: 12,
                volatility_multiplier: 10.0,
                decay_factor: 0.9,
                stable_period_hours: 2.0,
            },
            api: ApiConfig {
                gamma_api_base_url: "https://gamma-api.polymarket.com".to_string(),
                clob_api_base_url: "https://clob.polymarket.com".to_string(),
                max_retries: 3,
                timeout_seconds: 30,
                rate_limit_per_minute: 60,
                max_concurrent_requests: 16,
            },
            database: DatabaseConfig {
                url: "sqlite:///data/polymarket.db".to_string(),
                wal_mode: true,
                cache_size_mb: 64,
            },
            markets: MarketsConfig {
                min_volume: 1000.0,
                max_markets: 1000,
                categories: vec!["politics".to_string(), "economics".to_string()],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "logs/polymarket.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}
