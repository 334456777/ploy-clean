// 价格数据模型

use serde::{Deserialize, Serialize};

/// 价格点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Unix timestamp (秒)
    pub timestamp: i64,
    /// 价格/概率值 (0-1)
    pub price: f64,
    /// 交易量 (可选)
    pub volume: Option<f64>,
}

/// 变化点数据
#[derive(Debug, Clone)]
pub struct ChangePoint {
    /// 变化发生的时间戳
    pub timestamp: i64,
    /// 变化前的价格
    pub price_before: f64,
    /// 变化后的价格
    pub price_after: f64,
    /// 变化幅度 (绝对值)
    pub change_magnitude: f64,
    /// 置信度 (0-1)
    pub confidence: f64,
}

/// 采样间隔枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    OneMinute,
    FiveMinutes,
    OneHour,
    OneDay,
}

impl Interval {
    /// 获取 API 参数字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::FiveMinutes => "5m",
            Interval::OneHour => "1h",
            Interval::OneDay => "1d",
        }
    }

    /// 获取秒数
    pub fn as_seconds(&self) -> i64 {
        match self {
            Interval::OneMinute => 60,
            Interval::FiveMinutes => 300,
            Interval::OneHour => 3600,
            Interval::OneDay => 86400,
        }
    }
}

/// CLOB API 价格历史响应
#[derive(Debug, Clone, Deserialize)]
pub struct ClobPriceHistoryResponse {
    pub history: Option<Vec<ClobPricePoint>>,
    pub market: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClobPricePoint {
    #[serde(rename = "t")]
    pub timestamp: i64,
    #[serde(rename = "p")]
    pub price: f64,
}

impl From<ClobPricePoint> for PricePoint {
    fn from(clob: ClobPricePoint) -> Self {
        Self {
            timestamp: clob.timestamp, // API 已经返回秒级时间戳
            price: clob.price,
            volume: None,
        }
    }
}
