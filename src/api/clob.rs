// CLOB API 客户端

use super::ApiClient;
use crate::models::{PricePoint, Interval, ClobPriceHistoryResponse};
use anyhow::Result;
use std::collections::HashMap;

/// CLOB API 客户端
pub struct ClobClient {
    client: ApiClient,
}

impl ClobClient {
    /// 创建新的 CLOB API 客户端
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client: ApiClient::new(base_url)?.with_max_retries(3),
        })
    }

    /// 获取价格历史
    pub async fn fetch_price_history(
        &self,
        token_id: &str,
        interval: Interval,
    ) -> Result<Vec<PricePoint>> {
        let path = format!(
            "/prices-history?market={}&interval={}",
            token_id,
            interval.as_str()
        );

        match self.client.get::<ClobPriceHistoryResponse>(&path).await {
            Ok(response) => {
                let points = response
                    .history
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Ok(points)
            }
            Err(e) => {
                // 如果 API 返回 404 或空数据，返回空列表而不是错误
                let e_str = e.to_string();
                if e_str.contains("404") || e_str.contains("empty") {
                    Ok(Vec::new())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 批量获取多个 token 的当前价格
    pub async fn fetch_prices_batch(&self, token_ids: &[String]) -> Result<HashMap<String, f64>> {
        if token_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // CLOB API 支持批量查询
        let ids = token_ids.join(",");
        let path = format!("/prices?token_ids={}", ids);

        #[derive(Debug, serde::Deserialize)]
        struct PriceResponse {
            #[serde(rename = "token_id")]
            token_id: String,
            #[serde(rename = "price")]
            price: String,
        }

        match self.client.get::<Vec<PriceResponse>>(&path).await {
            Ok(response) => {
                let mut result = HashMap::new();
                for item in response {
                    if let Ok(price) = item.price.parse::<f64>() {
                        result.insert(item.token_id, price);
                    }
                }
                Ok(result)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch batch prices: {}", e);
                Ok(HashMap::new())
            }
        }
    }

    /// 获取单个 token 的当前价格
    pub async fn fetch_price(&self, token_id: &str) -> Result<Option<f64>> {
        let path = format!("/price?token_id={}", token_id);

        #[derive(Debug, serde::Deserialize)]
        struct PriceResponse {
            #[serde(rename = "price")]
            price: String,
        }

        match self.client.get::<PriceResponse>(&path).await {
            Ok(response) => Ok(response.price.parse().ok()),
            Err(_) => Ok(None),
        }
    }
}
