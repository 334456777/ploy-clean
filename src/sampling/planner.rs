// 采样计划器
// 根据变化点和策略生成采样计划

use crate::models::{PricePoint, ChangePoint};
use crate::sampling::{ChangePointDetector, AdaptiveSamplingStrategy, SamplingLevel};
use crate::api::ClobClient;
use crate::models::Interval;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// 采样计划器
#[derive(Clone)]
pub struct SamplingPlanner {
    clob_client: Arc<ClobClient>,
    detector: ChangePointDetector,
    strategy: AdaptiveSamplingStrategy,
}

impl SamplingPlanner {
    /// 创建新的采样计划器
    pub fn new(
        clob_client: Arc<ClobClient>,
        detector: ChangePointDetector,
        strategy: AdaptiveSamplingStrategy,
    ) -> Self {
        Self {
            clob_client,
            detector,
            strategy,
        }
    }

    /// 执行自适应采样
    ///
    /// # 流程
    /// 1. 获取粗粒度历史数据 (1d 间隔)
    /// 2. 检测变化点
    /// 3. 对变化点附近进行精细扫描 (1h 间隔)
    /// 4. 从最后变化点到当前密集采样 (1m-5m 间隔)
    pub async fn execute_adaptive_sampling(
        &self,
        token_id: &str,
    ) -> Result<Vec<PricePoint>> {
        let mut all_points = Vec::new();

        // 步骤 1: 获取粗粒度历史数据
        tracing::debug!("Fetching coarse-grained data for token {}", token_id);
        let coarse_data = self
            .clob_client
            .fetch_price_history(token_id, Interval::OneDay)
            .await?;

        if coarse_data.is_empty() {
            tracing::warn!("No coarse data available for token {}", token_id);
            return Ok(Vec::new());
        }

        // 步骤 2: 检测变化点
        let change_points = self.detector.detect_change_points(&coarse_data);
        tracing::debug!(
            "Detected {} change points for token {}",
            change_points.len(),
            token_id
        );

        // 步骤 3: 对变化点附近进行精细扫描
        for cp in &change_points {
            let fine_points = self.fetch_change_point_surroundings(token_id, cp).await?;
            all_points.extend(fine_points);
        }

        // 步骤 4: 从最后变化点到当前密集采样
        let last_timestamp = change_points
            .last()
            .map(|cp| cp.timestamp)
            .unwrap_or_else(|| coarse_data.first().map(|p| p.timestamp).unwrap_or(0));

        let recent_points = self.fetch_recent_data(token_id, last_timestamp).await?;
        all_points.extend(recent_points);

        // 添加粗粒度数据中没有被精细扫描覆盖的部分
        all_points.extend(coarse_data);

        Ok(all_points)
    }

    /// 获取变化点附近的精细数据
    async fn fetch_change_point_surroundings(
        &self,
        token_id: &str,
        change_point: &ChangePoint,
    ) -> Result<Vec<PricePoint>> {
        // 获取变化点前后各 12 小时的数据 (1h 间隔)
        let start = change_point.timestamp - 12 * 3600;
        let end = change_point.timestamp + 2 * 3600;

        // 注意: CLOB API 可能不支持时间范围参数，这里简化处理
        // 实际实现可能需要分批获取并过滤
        let fine_data = self
            .clob_client
            .fetch_price_history(token_id, Interval::OneHour)
            .await?;

        // 过滤出时间范围内的数据
        let filtered: Vec<PricePoint> = fine_data
            .into_iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .collect();

        Ok(filtered)
    }

    /// 获取最近的密集数据
    async fn fetch_recent_data(
        &self,
        token_id: &str,
        since_timestamp: i64,
    ) -> Result<Vec<PricePoint>> {
        // 根据当前波动率选择间隔
        // 这里简化处理，使用 5 分钟间隔
        let recent_data = self
            .clob_client
            .fetch_price_history(token_id, Interval::FiveMinutes)
            .await?;

        // 过滤出指定时间之后的数据
        let filtered: Vec<PricePoint> = recent_data
            .into_iter()
            .filter(|p| p.timestamp > since_timestamp)
            .collect();

        Ok(filtered)
    }

    /// 根据波动率生成采样计划
    pub fn generate_sampling_plan(
        &self,
        prices: &[PricePoint],
        change_points: &[ChangePoint],
    ) -> Vec<SamplingTask> {
        let mut tasks = Vec::new();

        if prices.is_empty() {
            return tasks;
        }

        let start = prices.first().unwrap().timestamp;
        let end = prices.last().unwrap().timestamp;

        // 如果没有变化点，使用单一采样级别
        if change_points.is_empty() {
            let volatility = self.calculate_volatility(prices);
            let level = self.strategy.get_sampling_level(volatility);
            let interval = self.strategy.get_interval_for_level(level);

            tasks.push(SamplingTask {
                start,
                end,
                interval,
                level,
            });
            return tasks;
        }

        // 根据变化点分割时间线，为每个段生成任务
        let mut prev_end = start;

        for cp in change_points {
            // 变化点之前使用稀疏采样
            let level = SamplingLevel::Low;
            let interval = self.strategy.get_interval_for_level(level);

            if cp.timestamp > prev_end {
                tasks.push(SamplingTask {
                    start: prev_end,
                    end: cp.timestamp,
                    interval,
                    level,
                });
            }

            // 变化点附近使用密集采样
            let high_level = SamplingLevel::High;
            let high_interval = self.strategy.get_interval_for_level(high_level);

            tasks.push(SamplingTask {
                start: cp.timestamp - 3600, // 变化前 1 小时
                end: cp.timestamp + 3600,   // 变化后 1 小时
                interval: high_interval,
                level: high_level,
            });

            prev_end = cp.timestamp + 3600;
        }

        // 最后一段到当前时间
        if end > prev_end {
            let level = SamplingLevel::Medium;
            let interval = self.strategy.get_interval_for_level(level);
            tasks.push(SamplingTask {
                start: prev_end,
                end,
                interval,
                level,
            });
        }

        tasks
    }

    /// 计算价格序列的波动率
    fn calculate_volatility(&self, prices: &[PricePoint]) -> f64 {
        if prices.len() <= 1 {
            return 0.0;
        }

        let values: Vec<f64> = prices.iter().map(|p| p.price).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / values.len() as f64;
        variance.sqrt()
    }
}

/// 采样任务
#[derive(Debug, Clone)]
pub struct SamplingTask {
    pub start: i64,
    pub end: i64,
    pub interval: Duration,
    pub level: SamplingLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sampling_plan_no_changes() {
        // 简化测试，需要 mock detector 和 strategy
    }
}
