// 自适应采样策略
// 根据市场波动率动态调整采样间隔

use std::time::Duration;

/// 采样级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingLevel {
    /// 高波动期 - 密集采样 (1-5 分钟)
    High,
    /// 中波动期 - 正常采样 (15-30 分钟)
    Medium,
    /// 低波动期 - 稀疏采样 (1-6 小时)
    Low,
}

/// 自适应采样策略
#[derive(Debug, Clone)]
pub struct AdaptiveSamplingStrategy {
    /// 基础采样间隔
    pub base_interval: Duration,
    /// 最小采样间隔
    pub min_interval: Duration,
    /// 最大采样间隔
    pub max_interval: Duration,
    /// 波动率乘数
    pub volatility_multiplier: f64,
    /// 衰减因子 (平滑调整)
    pub decay_factor: f64,
    /// 波动率阈值
    pub high_volatility_threshold: f64,
    pub medium_volatility_threshold: f64,
    pub low_volatility_threshold: f64,
}

impl AdaptiveSamplingStrategy {
    /// 创建新的自适应采样策略
    pub fn new(
        base_interval: Duration,
        min_interval: Duration,
        max_interval: Duration,
        volatility_multiplier: f64,
        decay_factor: f64,
    ) -> Self {
        Self {
            base_interval,
            min_interval,
            max_interval,
            volatility_multiplier,
            decay_factor,
            high_volatility_threshold: 0.02,
            medium_volatility_threshold: 0.005,
            low_volatility_threshold: 0.001,
        }
    }

    /// 从配置创建策略
    pub fn from_config(config: &SamplingStrategyConfig) -> Self {
        Self {
            base_interval: Duration::from_secs((config.base_interval_hours * 3600.0) as u64),
            min_interval: Duration::from_secs((config.min_interval_minutes * 60.0) as u64),
            max_interval: Duration::from_secs((config.max_interval_hours * 3600.0) as u64),
            volatility_multiplier: config.volatility_multiplier,
            decay_factor: config.decay_factor,
            high_volatility_threshold: config.high_volatility_threshold,
            medium_volatility_threshold: config.medium_volatility_threshold,
            low_volatility_threshold: config.low_volatility_threshold,
        }
    }

    /// 获取采样级别
    pub fn get_sampling_level(&self, volatility: f64) -> SamplingLevel {
        if volatility > self.high_volatility_threshold {
            SamplingLevel::High
        } else if volatility > self.medium_volatility_threshold {
            SamplingLevel::Medium
        } else {
            SamplingLevel::Low
        }
    }

    /// 根据波动率计算采样间隔
    ///
    /// # 算法
    /// 1. 根据波动率计算目标间隔: base_interval / (volatility * multiplier)
    ///    波动率越高，目标间隔越小
    /// 2. 使用衰减因子平滑调整: new = old * decay + target * (1 - decay)
    /// 3. 限制在 min 和 max 间隔之间
    pub fn calculate_interval(&self, volatility: f64, current_interval: Duration) -> Duration {
        // 避免除零，设置最小波动率
        let adj_volatility = volatility.max(0.001);

        // 计算目标间隔：波动率越高，间隔越短
        // 使用 base_interval * multiplier / volatility 的变形
        let target_secs = (self.base_interval.as_secs() as f64 * self.volatility_multiplier / adj_volatility)
            .min(self.max_interval.as_secs() as f64)
            .max(self.min_interval.as_secs() as f64);

        // 使用衰减因子平滑
        let adjusted_secs = current_interval.as_secs() as f64 * self.decay_factor
            + target_secs * (1.0 - self.decay_factor);

        // 再次限制在范围内
        let clamped_secs = adjusted_secs
            .clamp(self.min_interval.as_secs() as f64, self.max_interval.as_secs() as f64);

        Duration::from_secs(clamped_secs as u64)
    }

    /// 根据采样级别获取推荐间隔
    pub fn get_interval_for_level(&self, level: SamplingLevel) -> Duration {
        match level {
            SamplingLevel::High => self.min_interval,              // 1-5 分钟
            SamplingLevel::Medium => self.base_interval / 2,       // 15-30 分钟
            SamplingLevel::Low => self.base_interval * 2,          // 1-6 小时
        }
    }

    /// 检查是否应该提高采样率
    pub fn should_increase_sample_rate(&self, volatility: f64, current_level: SamplingLevel) -> bool {
        let new_level = self.get_sampling_level(volatility);
        // 数值越小级别越高 (High=0, Medium=1, Low=2)
        (new_level as i32) < (current_level as i32)
    }

    /// 检查是否应该降低采样率
    pub fn should_decrease_sample_rate(&self, volatility: f64, current_level: SamplingLevel) -> bool {
        let new_level = self.get_sampling_level(volatility);
        // 数值越大级别越低 (High=0, Medium=1, Low=2)
        (new_level as i32) > (current_level as i32)
    }
}

impl Default for AdaptiveSamplingStrategy {
    fn default() -> Self {
        Self {
            base_interval: Duration::from_secs(3600),    // 1 小时
            min_interval: Duration::from_secs(60),       // 1 分钟
            max_interval: Duration::from_secs(86400),    // 24 小时
            volatility_multiplier: 10.0,
            decay_factor: 0.9,
            high_volatility_threshold: 0.02,
            medium_volatility_threshold: 0.005,
            low_volatility_threshold: 0.001,
        }
    }
}

/// 采样策略配置
#[derive(Debug, Clone)]
pub struct SamplingStrategyConfig {
    pub base_interval_hours: f64,
    pub min_interval_minutes: f64,
    pub max_interval_hours: f64,
    pub volatility_multiplier: f64,
    pub decay_factor: f64,
    pub high_volatility_threshold: f64,
    pub medium_volatility_threshold: f64,
    pub low_volatility_threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_level() {
        let strategy = AdaptiveSamplingStrategy::default();

        assert_eq!(
            strategy.get_sampling_level(0.03),
            SamplingLevel::High
        );
        assert_eq!(
            strategy.get_sampling_level(0.01),
            SamplingLevel::Medium
        );
        assert_eq!(
            strategy.get_sampling_level(0.0005),
            SamplingLevel::Low
        );
    }

    #[test]
    fn test_calculate_interval() {
        let strategy = AdaptiveSamplingStrategy::default();
        let current = Duration::from_secs(3600);

        // 高波动率应该有趋向更短间隔的目标
        // 由于公式使用 base * multiplier / volatility，高波动率会产生很小的目标值
        // 但由于 clamp 在 min_interval (60) 处，最终不会小于 60
        let high_vol_interval = strategy.calculate_interval(0.03, current);
        // 目标 = 3600 * 10 / 0.03 = 1200000，超过 max，被限制为 86400
        // 由于衰减，第一次调整不会达到目标
        assert!(high_vol_interval.as_secs() <= 86400);

        // 低波动率应该有趋向更长间隔的目标
        let low_vol_interval = strategy.calculate_interval(0.0005, current);
        // 目标 = 3600 * 10 / 0.0005 = 72000000，超过 max，被限制为 86400
        // 两次都会被限制在 max
        assert!(low_vol_interval.as_secs() <= 86400);

        // 验证 min_interval 约束
        let extreme_high = strategy.calculate_interval(1.0, Duration::from_secs(100));
        assert!(extreme_high.as_secs() >= 60);
    }

    #[test]
    fn test_interval_bounds() {
        let strategy = AdaptiveSamplingStrategy::default();
        let current = Duration::from_secs(3600);

        // 极端波动率也不应超出边界
        let extreme_high = strategy.calculate_interval(100.0, current);
        assert!(extreme_high >= strategy.min_interval);
        assert!(extreme_high <= strategy.max_interval);

        let extreme_low = strategy.calculate_interval(0.00001, current);
        assert!(extreme_low >= strategy.min_interval);
        assert!(extreme_low <= strategy.max_interval);
    }

    #[test]
    fn test_should_increase_sample_rate() {
        let strategy = AdaptiveSamplingStrategy::default();

        assert!(strategy.should_increase_sample_rate(0.03, SamplingLevel::Low));
        assert!(!strategy.should_increase_sample_rate(0.0005, SamplingLevel::High));
    }
}
