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

    /// 搜索市场，支持限制最大返回数量和扫描深度
    /// 注意: Gamma API 不支持服务端关键词搜索，采用客户端全量扫描 + 过滤实现
    ///
    /// # 参数
    /// - `query`: 搜索关键词（空格分隔，AND 逻辑）
    /// - `limit`: 最大返回结果数
    /// - `max_pages`: 最大扫描页数（每页 100 个市场）
    /// - `on_progress`: 进度回调 (当前页, 已找到结果数)
    pub async fn search_markets<F>(
        &self,
        query: &str,
        limit: usize,
        max_pages: usize,
        on_progress: F,
    ) -> Result<Vec<Market>>
    where
        F: Fn(usize, usize),
    {
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();
        let mut results = Vec::new();
        let batch_size = 100;

        for page in 0..max_pages {
            let offset = page * batch_size;
            let path = format!(
                "/markets?limit={}&offset={}&active=true&closed=false",
                batch_size, offset
            );
            let gamma_markets: Vec<GammaMarket> = self.client.get(&path).await?;

            if gamma_markets.is_empty() {
                on_progress(page + 1, results.len());
                break;
            }

            for m in gamma_markets {
                let question_lower = m.question.to_lowercase();
                let slug_lower = m.slug.as_deref().unwrap_or("").to_lowercase();
                let desc_lower = m.description.as_deref().unwrap_or("").to_lowercase();

                // 所有关键词都必须匹配（AND 逻辑）
                let matched = keywords.iter().all(|kw| {
                    question_lower.contains(kw)
                        || slug_lower.contains(kw)
                        || desc_lower.contains(kw)
                });

                if matched {
                    results.push(Market::from(m));
                    if results.len() >= limit {
                        on_progress(page + 1, results.len());
                        return Ok(results);
                    }
                }
            }

            on_progress(page + 1, results.len());
        }

        Ok(results)
    }
}
