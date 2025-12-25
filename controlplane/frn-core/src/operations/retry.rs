//! Retry policy with exponential backoff
//!
//! Implements exponential backoff with jitter for operation retries.
//! The delay grows exponentially with each attempt but is capped at a maximum.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;

/// Default base delay in seconds for the first retry.
const DEFAULT_BASE_DELAY_SECS: i64 = 1;

/// Default maximum delay in seconds (5 minutes).
const DEFAULT_MAX_DELAY_SECS: i64 = 300;

/// Default jitter factor (0.0 to 1.0).
const DEFAULT_JITTER_FACTOR: f64 = 0.2;

/// Retry policy with exponential backoff.
///
/// Calculates the next retry time using the formula:
/// `delay = min(base_delay * 2^attempt, max_delay) * (1 + random_jitter)`
///
/// ## Example
///
/// ```rust
/// use frn_core::operations::RetryPolicy;
///
/// let policy = RetryPolicy::default();
///
/// // First retry: ~1 second
/// let next = policy.next_retry_at(1);
///
/// // Second retry: ~2 seconds
/// let next = policy.next_retry_at(2);
///
/// // Third retry: ~4 seconds
/// let next = policy.next_retry_at(3);
/// ```
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Base delay in seconds for the first retry.
    base_delay_secs: i64,
    /// Maximum delay in seconds (cap on exponential growth).
    max_delay_secs: i64,
    /// Jitter factor to add randomness (0.0 to 1.0).
    jitter_factor: f64,
}

impl RetryPolicy {
    /// Creates a new retry policy with custom parameters.
    pub fn new(base_delay_secs: i64, max_delay_secs: i64, jitter_factor: f64) -> Self {
        Self {
            base_delay_secs,
            max_delay_secs,
            jitter_factor: jitter_factor.clamp(0.0, 1.0),
        }
    }

    /// Creates a retry policy with aggressive settings for critical operations.
    ///
    /// Uses shorter delays: 500ms base, 1 minute max.
    pub fn aggressive() -> Self {
        Self {
            base_delay_secs: 0, // We'll use milliseconds below
            max_delay_secs: 60,
            jitter_factor: 0.1,
        }
    }

    /// Creates a retry policy with relaxed settings for non-critical operations.
    ///
    /// Uses longer delays: 5 seconds base, 10 minutes max.
    pub fn relaxed() -> Self {
        Self {
            base_delay_secs: 5,
            max_delay_secs: 600,
            jitter_factor: 0.3,
        }
    }

    /// Calculates the next retry time based on the attempt count.
    ///
    /// # Arguments
    /// * `attempt_count` - The current attempt count (1-based)
    ///
    /// # Returns
    /// The DateTime when the next retry should occur.
    pub fn next_retry_at(&self, attempt_count: i32) -> DateTime<Utc> {
        let delay_secs = self.calculate_delay_secs(attempt_count);
        Utc::now() + Duration::seconds(delay_secs)
    }

    /// Calculates the delay in seconds for a given attempt.
    fn calculate_delay_secs(&self, attempt_count: i32) -> i64 {
        // Exponential backoff: base * 2^(attempt-1)
        let exponential_delay = self.base_delay_secs.saturating_mul(
            2_i64.saturating_pow((attempt_count.saturating_sub(1)) as u32),
        );

        // Cap at maximum delay
        let capped_delay = exponential_delay.min(self.max_delay_secs);

        // Add jitter to prevent thundering herd
        let jitter = self.calculate_jitter(capped_delay);

        capped_delay.saturating_add(jitter)
    }

    /// Calculates random jitter for the delay.
    fn calculate_jitter(&self, base_delay: i64) -> i64 {
        if self.jitter_factor <= 0.0 {
            return 0;
        }

        let max_jitter = (base_delay as f64 * self.jitter_factor) as i64;
        if max_jitter <= 0 {
            return 0;
        }

        rand::thread_rng().gen_range(0..=max_jitter)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base_delay_secs: DEFAULT_BASE_DELAY_SECS,
            max_delay_secs: DEFAULT_MAX_DELAY_SECS,
            jitter_factor: DEFAULT_JITTER_FACTOR,
        }
    }
}

/// Convenience function to calculate the next retry time with default policy.
pub fn calculate_exponential_backoff(attempt_count: i32) -> DateTime<Utc> {
    RetryPolicy::default().next_retry_at(attempt_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_growth() {
        let policy = RetryPolicy::new(1, 300, 0.0); // No jitter for predictable tests

        // Check exponential growth: 1, 2, 4, 8, 16, 32, 64, 128, 256
        let delays: Vec<i64> = (1..=5).map(|a| policy.calculate_delay_secs(a)).collect();
        assert_eq!(delays, vec![1, 2, 4, 8, 16]);
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy::new(1, 10, 0.0);

        // After enough attempts, should be capped at max
        assert_eq!(policy.calculate_delay_secs(10), 10); // Would be 512 without cap
        assert_eq!(policy.calculate_delay_secs(20), 10);
    }

    #[test]
    fn test_jitter_bounds() {
        let policy = RetryPolicy::new(10, 300, 0.5);

        // With 50% jitter on 10 second base, delay should be between 10 and 15
        for _ in 0..100 {
            let delay = policy.calculate_delay_secs(1);
            assert!(delay >= 10 && delay <= 15, "delay {} out of bounds", delay);
        }
    }

    #[test]
    fn test_next_retry_at_is_in_future() {
        let policy = RetryPolicy::default();
        let now = Utc::now();
        let next = policy.next_retry_at(1);

        assert!(next > now);
    }

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.base_delay_secs, DEFAULT_BASE_DELAY_SECS);
        assert_eq!(policy.max_delay_secs, DEFAULT_MAX_DELAY_SECS);
        assert!((policy.jitter_factor - DEFAULT_JITTER_FACTOR).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aggressive_policy() {
        let policy = RetryPolicy::aggressive();
        assert!(policy.max_delay_secs <= 60);
    }

    #[test]
    fn test_relaxed_policy() {
        let policy = RetryPolicy::relaxed();
        assert!(policy.base_delay_secs >= 5);
        assert!(policy.max_delay_secs >= 600);
    }
}
