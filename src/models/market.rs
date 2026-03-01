// 市场数据模型

use serde::{Deserialize, Serialize};

/// Polymarket 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// 市场唯一标识符
    pub id: String,
    /// 市场问题/描述
    pub question: String,
    /// 条件 ID (Condition ID)
    pub condition_id: String,
    /// URL slug
    pub slug: String,
    /// 可能的结果选项
    pub outcomes: Vec<String>,
    /// 交易量
    pub volume: Option<f64>,
    /// 流动性
    pub liquidity: Option<f64>,
    /// 是否活跃
    pub active: bool,
    /// 是否已关闭
    pub closed: bool,
    /// 结束日期 (ISO 8601 日期字符串)
    pub end_date: Option<String>,
    /// 市场分类
    pub category: Option<String>,
    /// CLOB token IDs (用于获取价格历史)
    pub clob_token_ids: Option<Vec<String>>,
}

/// Gamma API 返回的市场数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaMarket {
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(rename = "question")]
    pub question: String,
    #[serde(rename = "slug")]
    pub slug: Option<String>,
    #[serde(rename = "outcomes")]
    pub outcomes: String,  // API 返回 JSON 字符串，如 "[\"Yes\", \"No\"]"
    #[serde(rename = "outcomePrices")]
    pub outcome_prices: Option<String>,  // API 返回 JSON 字符串
    #[serde(rename = "volume")]
    pub volume: Option<String>,
    #[serde(rename = "liquidity")]
    pub liquidity: Option<String>,
    #[serde(rename = "active")]
    pub active: Option<bool>,
    #[serde(rename = "closed")]
    pub closed: Option<bool>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,  // ISO 8601 日期字符串
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(rename = "clobTokenIds")]
    pub clob_token_ids: Option<String>,
}

impl From<GammaMarket> for Market {
    fn from(gamma: GammaMarket) -> Self {
        // 解析 outcomes JSON 字符串
        let outcomes: Vec<String> = serde_json::from_str(&gamma.outcomes)
            .unwrap_or_else(|_| vec![gamma.outcomes.clone()]);

        Self {
            id: gamma.id.unwrap_or_else(|| gamma.condition_id.clone()),
            question: gamma.question,
            condition_id: gamma.condition_id,
            slug: gamma.slug.unwrap_or_default(),
            outcomes,
            volume: gamma.volume.and_then(|v| v.parse().ok()),
            liquidity: gamma.liquidity.and_then(|l| l.parse().ok()),
            active: gamma.active.unwrap_or(true),
            closed: gamma.closed.unwrap_or(false),
            end_date: gamma.end_date,
            category: None,
            clob_token_ids: gamma.clob_token_ids.and_then(|ids| serde_json::from_str(&ids).ok()),
        }
    }
}
