//! One retry policy for every queue: an error is retryable or terminal, a
//! retry waits an exponentially growing, jittered, capped delay, and the
//! attempt count is bounded so a job that keeps failing ends even if its
//! queue lifetime somehow does not.
use mail_kernel::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Retryable,
    Terminal,
}

/// Malformed, unknown or refused targets never succeed on retry; storage
/// and contention failures do.
pub fn classify(error: Error) -> Class {
    match error {
        Error::Invalid | Error::NotFound | Error::Forbidden => Class::Terminal,
        Error::Conflict | Error::OverQuota | Error::Unavailable | Error::Busy => Class::Retryable,
    }
}

/// Attempts after which a job is terminal regardless of its errors. The
/// five-day queue lifetime ends a job first under the normal schedule.
pub const MAX_ATTEMPTS: u32 = 40;
const MIN_DELAY_SECS: i64 = 60;
const MAX_DELAY_SECS: i64 = 6 * 60 * 60;

/// Seconds before attempt `attempt + 1` (1-based `attempt` just finished):
/// 60 s doubling to a 6 h cap, plus up to a quarter of the step as jitter
/// from `entropy` so retries of many jobs do not align.
pub fn delay_secs(attempt: u32, entropy: u64) -> i64 {
    let base = (MIN_DELAY_SECS << attempt.saturating_sub(1).min(9)).min(MAX_DELAY_SECS);
    base + (entropy % (base / 4).max(1) as u64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_double_from_a_minute_to_a_six_hour_cap_with_bounded_jitter() {
        assert_eq!(delay_secs(1, 0), 60);
        assert_eq!(delay_secs(2, 0), 120);
        assert_eq!(delay_secs(10, 0), MAX_DELAY_SECS);
        assert_eq!(delay_secs(30, 0), MAX_DELAY_SECS);
        assert!(delay_secs(1, u64::MAX) < 75 && delay_secs(1, 14) == 74);
        assert_eq!(classify(Error::Busy), Class::Retryable);
        assert_eq!(classify(Error::NotFound), Class::Terminal);
    }
}
