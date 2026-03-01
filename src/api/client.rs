// 通用 HTTP 客户端

use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// 通用 API 客户端
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    max_retries: u32,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.into(),
            max_retries: 3,
        })
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 发送 GET 请求并处理重试
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response.json().await?);
                    } else if response.status().as_u16() == 429 {
                        // 速率限制，使用指数退避
                        let delay = 2_u64.pow(attempt) * 100;
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        anyhow::bail!("API error: {} - {}", status, text);
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        let delay = 2_u64.pow(attempt) * 100;
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.map(|e| e.into()).unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
