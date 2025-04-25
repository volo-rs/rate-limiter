/// A token bucket rate limiter implementation bases on lazy-update strategy.
use crate::RateLimiter;
use std::cmp::min;
use std::sync::Mutex;

pub struct TokenBucketRateLimiter {
    status: Mutex<TokenBucketRateLimiterStatus>,
    clock: quanta::Clock,
    base_instant: quanta::Instant,
}

pub struct TokenBucketRateLimiterStatus {
    rate_in_seconds: i64,
    capacity: i64,

    last_updated_timestamp_in_seconds: i64,
    tokens: i64,
}

impl RateLimiter for TokenBucketRateLimiter {
    fn acquire(&self) -> Result<(), ()> {
        let now = self.clock.now().duration_since(self.base_instant).as_secs() as i64;

        let mut guard = self
            .status
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        
        let last_updated = guard.last_updated_timestamp_in_seconds;
        
        let tokens = min(
            guard.capacity,
            guard.tokens + (now - last_updated) * guard.rate_in_seconds,
        );

        if tokens <= 0 {
            return Err(());
        }

        guard.last_updated_timestamp_in_seconds = now;
        guard.tokens = tokens - 1;

        Ok(())
    }
}

impl TokenBucketRateLimiter {
    pub fn new(rate_in_seconds: i64, capacity: i64) -> Self {
        let clock = quanta::Clock::new();
        let base_instant = clock.now();

        Self {
            status: Mutex::new(TokenBucketRateLimiterStatus {
                rate_in_seconds,
                capacity,
                last_updated_timestamp_in_seconds: 0,
                tokens: 0,
            }),
            clock,
            base_instant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_rate_limiter() {
        let limiter = TokenBucketRateLimiter::new(1, 1);
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(limiter.acquire(), Ok(()));
        assert_eq!(limiter.acquire(), Err(()));
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(limiter.acquire(), Ok(()));
        assert_eq!(limiter.acquire(), Err(()));
    }
}
