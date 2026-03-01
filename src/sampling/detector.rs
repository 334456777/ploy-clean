// 变化点检测器 - 核心算法
// 使用滑动窗口 + Z-score 统计检验检测价格序列中的变化点

use crate::models::{PricePoint, ChangePoint};

/// 变化点检测器
#[derive(Debug, Clone)]
pub struct ChangePointDetector {
    /// 滑动窗口大小 (数据点数量)
    pub window_size: usize,
    /// 最小样本数
    pub min_samples: usize,
    /// 置信水平 (0-1)
    pub confidence_level: f64,
    /// Z-score 阈值 (对应置信水平)
    pub z_score_threshold: f64,
}

impl ChangePointDetector {
    /// 创建新的变化点检测器
    pub fn new(window_size: usize, min_samples: usize, confidence_level: f64) -> Self {
        // 根据置信水平计算 Z-score 阈值
        // 95% 置信度 ≈ 1.96, 99% 置信度 ≈ 2.58
        let z_score_threshold = match confidence_level {
            cl if cl >= 0.99 => 2.58,
            cl if cl >= 0.95 => 1.96,
            cl if cl >= 0.90 => 1.645,
            _ => 1.96,
        };

        Self {
            window_size,
            min_samples,
            confidence_level,
            z_score_threshold,
        }
    }

    /// 从配置创建检测器
    pub fn from_config(config: &SamplingConfig) -> Self {
        Self::new(
            config.change_point_detection_window_size,
            config.change_point_min_samples,
            config.change_point_confidence_level,
        )
    }

    /// 检测价格序列中的变化点
    ///
    /// # 算法流程
    /// 1. 使用滑动窗口遍历价格序列
    /// 2. 对每个位置，比较前后窗口的统计特性
    /// 3. 如果 Z-score 超过阈值，标记为变化点
    /// 4. 合并相邻的变化点
    pub fn detect_change_points(&self, prices: &[PricePoint]) -> Vec<ChangePoint> {
        if prices.len() < self.min_samples * 2 {
            return Vec::new();
        }

        let mut change_points = Vec::new();
        let mut i = self.window_size;

        // 滑动窗口检测
        while i + self.window_size < prices.len() {
            let before_window = &prices[i - self.window_size..i];
            let after_window = &prices[i..i + self.window_size];

            if let Some(cp) = self.test_change(before_window, after_window, prices[i].timestamp) {
                change_points.push(cp);
                // 跳过已检测的区域，避免重复
                i += self.window_size;
            } else {
                i += 1;
            }
        }

        // 合并相邻的变化点
        self.merge_nearby_change_points(change_points)
    }

    /// 测试两个窗口之间是否有显著变化
    fn test_change(
        &self,
        before: &[PricePoint],
        after: &[PricePoint],
        timestamp: i64,
    ) -> Option<ChangePoint> {
        // 计算均值
        let mean_before: f64 = before.iter().map(|p| p.price).sum::<f64>() / before.len() as f64;
        let mean_after: f64 = after.iter().map(|p| p.price).sum::<f64>() / after.len() as f64;

        // 计算标准差
        let std_before = self.calculate_std(before, mean_before);
        let std_after = self.calculate_std(after, mean_after);

        // 合并标准差 (pooled standard deviation)
        let pooled_std = ((std_before.powi(2) + std_after.powi(2)) / 2.0).sqrt();

        if pooled_std == 0.0 {
            return None;
        }

        // Z-score 检验
        let z_score = (mean_after - mean_before).abs() / pooled_std;

        if z_score >= self.z_score_threshold {
            Some(ChangePoint {
                timestamp,
                price_before: mean_before,
                price_after: mean_after,
                change_magnitude: (mean_after - mean_before).abs(),
                confidence: self.confidence_level,
            })
        } else {
            None
        }
    }

    /// 计算标准差
    fn calculate_std(&self, prices: &[PricePoint], mean: f64) -> f64 {
        if prices.len() <= 1 {
            return 0.0;
        }
        let variance = prices
            .iter()
            .map(|p| (p.price - mean).powi(2))
            .sum::<f64>()
            / (prices.len() as f64 - 1.0);
        variance.sqrt()
    }

    /// 合并相邻的变化点
    fn merge_nearby_change_points(&self, mut points: Vec<ChangePoint>) -> Vec<ChangePoint> {
        if points.len() <= 1 {
            return points;
        }

        // 按时间戳排序
        points.sort_by_key(|p| p.timestamp);

        let mut merged = Vec::new();
        let mut current = points.remove(0);

        for point in points {
            // 如果两个变化点时间差小于窗口大小的一半，合并它们
            if point.timestamp - current.timestamp < (self.window_size as i64 / 2) {
                // 保留变化幅度更大的点
                if point.change_magnitude > current.change_magnitude {
                    current = point;
                }
            } else {
                merged.push(current);
                current = point;
            }
        }
        merged.push(current);

        merged
    }
}

/// 采样配置
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub change_point_detection_window_size: usize,
    pub change_point_min_samples: usize,
    pub change_point_confidence_level: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_price_points(prices: &[f64], start_timestamp: i64) -> Vec<PricePoint> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| PricePoint {
                timestamp: start_timestamp + i as i64 * 3600,
                price: p,
                volume: None,
            })
            .collect()
    }

    #[test]
    fn test_detect_change_points() {
        let detector = ChangePointDetector::new(5, 10, 0.95);

        // 创建有明显变化的价格序列
        // 前半部分稳定在 0.5，后半部分跳到 0.7
        let mut prices = vec![0.5; 20];
        prices.extend(vec![0.7; 20]);

        let price_points = create_price_points(&prices, 1000000);
        let change_points = detector.detect_change_points(&price_points);

        // 应该检测到大约在位置 20 的变化点
        assert!(!change_points.is_empty());
        let main_change = &change_points[0];
        assert!((main_change.price_before - 0.5).abs() < 0.1);
        assert!((main_change.price_after - 0.7).abs() < 0.1);
    }

    #[test]
    fn test_no_change_in_stable_series() {
        let detector = ChangePointDetector::new(5, 10, 0.95);

        // 稳定价格序列
        let prices = vec![0.5; 30];
        let price_points = create_price_points(&prices, 1000000);
        let change_points = detector.detect_change_points(&price_points);

        assert!(change_points.is_empty());
    }

    #[test]
    fn test_z_score_threshold() {
        let detector = ChangePointDetector::new(5, 10, 0.95);
        assert!((detector.z_score_threshold - 1.96).abs() < 0.01);

        let detector_99 = ChangePointDetector::new(5, 10, 0.99);
        assert!((detector_99.z_score_threshold - 2.58).abs() < 0.01);
    }
}
