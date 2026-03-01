// Gamma API 客户端

use super::ApiClient;
use crate::models::{Market, GammaMarket};
use anyhow::Result;

/// Gamma API 客户端
pub struct GammaClient {
    client: ApiClient,
}

impl GammaClient {
    /// 创建新的 Gamma API 客户端
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client: ApiClient::new(base_url)?.with_max_retries(3),
        })
    }

    /// 获取所有活跃市场
    pub async fn fetch_markets(&self) -> Result<Vec<Market>> {
        self.fetch_markets_batch(0, 100).await
    }

    /// 分页获取市场列表
    pub async fn fetch_markets_batch(&self, offset: usize, limit: usize) -> Result<Vec<Market>> {
        // 获取活跃且未关闭的市场
        let path = format!("/markets?limit={}&offset={}&active=true&closed=false", limit, offset);
        let gamma_markets: Vec<GammaMarket> = self.client.get(&path).await?;

        Ok(gamma_markets
            .into_iter()
            .filter_map(|m| {
                // 过滤掉无效数据
                if m.question.is_empty() {
                    None
                } else {
                    Some(m.into())
                }
            })
            .collect())
    }

    /// 获取单个市场详情
    pub async fn fetch_market(&self, market_id: &str) -> Result<Market> {
        let path = format!("/markets/{}", market_id);
        let gamma_market: GammaMarket = self.client.get(&path).await?;
        Ok(gamma_market.into())
    }

    /// 搜索市场，支持限制最大返回数量
    pub async fn search_markets(&self, query: &str, limit: usize) -> Result<Vec<Market>> {
        let path = format!("/markets?query={}&limit={}", urlencoding::encode(query), limit);
        let gamma_markets: Vec<GammaMarket> = self.client.get(&path).await?;

        Ok(gamma_markets
            .into_iter()
            .take(limit)
            .map(Into::into)
            .collect())
    }
}
