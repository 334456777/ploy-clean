// 时间处理工具

use chrono::{DateTime, TimeZone, Utc};

/// Unix timestamp (秒) 转换为 DateTime
pub fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).unwrap()
}

/// DateTime 转换为 Unix timestamp (秒)
pub fn datetime_to_timestamp(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// 获取当前时间戳 (秒)
pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// 格式化时间戳为可读字符串
pub fn format_timestamp(ts: i64) -> String {
    let dt = timestamp_to_datetime(ts);
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// 计算两个时间戳之间的时间差 (秒)
pub fn time_diff_seconds(start: i64, end: i64) -> i64 {
    end.abs_diff(start) as i64
}

/// 将时间戳按指定间隔向下对齐
pub fn floor_to_interval(timestamp: i64, interval_seconds: i64) -> i64 {
    (timestamp / interval_seconds) * interval_seconds
}

/// 将时间戳按指定间隔向上对齐
pub fn ceil_to_interval(timestamp: i64, interval_seconds: i64) -> i64 {
    ((timestamp + interval_seconds - 1) / interval_seconds) * interval_seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_to_interval() {
        assert_eq!(floor_to_interval(3661, 3600), 3600);
        assert_eq!(floor_to_interval(3600, 3600), 3600);
    }

    #[test]
    fn test_ceil_to_interval() {
        assert_eq!(ceil_to_interval(3661, 3600), 7200);
        assert_eq!(ceil_to_interval(3600, 3600), 3600);
    }

    #[test]
    fn test_time_diff() {
        assert_eq!(time_diff_seconds(1000, 2000), 1000);
    }
}
