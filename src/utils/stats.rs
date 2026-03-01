// 统计算法工具

use crate::models::PricePoint;

/// 计算均值
pub fn calculate_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// 计算标准差
pub fn calculate_std(values: &[f64], mean: f64) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / (values.len() as f64 - 1.0);
    variance.sqrt()
}

/// 计算波动率 (使用价格序列)
pub fn calculate_volatility(prices: &[PricePoint]) -> f64 {
    if prices.len() <= 1 {
        return 0.0;
    }

    let values: Vec<f64> = prices.iter().map(|p| p.price).collect();
    let mean = calculate_mean(&values);
    calculate_std(&values, mean)
}

/// 计算变化率 (一阶导数)
pub fn calculate_velocity(prices: &[PricePoint]) -> Vec<f64> {
    if prices.len() <= 1 {
        return Vec::new();
    }

    prices
        .windows(2)
        .map(|w| {
            let price_change = w[1].price - w[0].price;
            let time_change = (w[1].timestamp - w[0].timestamp) as f64;
            if time_change > 0.0 {
                price_change / time_change
            } else {
                0.0
            }
        })
        .collect()
}

/// 计算加速度 (二阶导数)
pub fn calculate_acceleration(prices: &[PricePoint]) -> Vec<f64> {
    let velocities = calculate_velocity(prices);
    if velocities.len() <= 1 {
        return Vec::new();
    }

    velocities
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

/// 计算 Z-score
pub fn calculate_z_score(value: f64, mean: f64, std: f64) -> f64 {
    if std == 0.0 {
        0.0
    } else {
        (value - mean) / std
    }
}

/// 计算两个窗口之间的统计差异 (Z-score 检验)
pub fn calculate_window_difference(before: &[PricePoint], after: &[PricePoint]) -> f64 {
    if before.is_empty() || after.is_empty() {
        return 0.0;
    }

    let before_prices: Vec<f64> = before.iter().map(|p| p.price).collect();
    let after_prices: Vec<f64> = after.iter().map(|p| p.price).collect();

    let mean_before = calculate_mean(&before_prices);
    let mean_after = calculate_mean(&after_prices);
    let std_before = calculate_std(&before_prices, mean_before);
    let std_after = calculate_std(&after_prices, mean_after);

    // 合并标准差
    let pooled_std = ((std_before.powi(2) + std_after.powi(2)) / 2.0).sqrt();

    if pooled_std == 0.0 {
        return 0.0;
    }

    // Z-score
    (mean_after - mean_before) / pooled_std
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mean() {
        assert!((calculate_mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_std() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mean = calculate_mean(&values);
        let std = calculate_std(&values, mean);
        assert!((std - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_z_score() {
        assert!((calculate_z_score(15.0, 10.0, 2.0) - 2.5).abs() < f64::EPSILON);
    }
}
