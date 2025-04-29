/// A token bucket rate limiter implementation bases on lazy-update strategy with CPU sharding.
use crate::{RateLimiter, TokenBucketRateLimiter};
use std::cmp::min;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::thread::available_parallelism;

pub struct ShardingTokenBucketRateLimiter {
    index: AtomicUsize,
    shards: Vec<Arc<TokenBucketRateLimiter>>,
}

impl RateLimiter for ShardingTokenBucketRateLimiter {
    fn acquire(&self) -> Result<(), ()> {
        let index = self.index.fetch_add(1, Relaxed) % self.shards.len();
        let limiter = self.shards[index].clone();
        limiter.acquire()
    }
}

impl ShardingTokenBucketRateLimiter {
    pub fn new(rate_in_seconds: i64, capacity: i64) -> Self {
        // use the number of physical CPUs to avoid burst computing 
        // that causes the maximum parallelism to exceed the cgroups limit
        let mut parallelism = num_cpus::get_physical() as i64;
        parallelism = min(parallelism, rate_in_seconds);
        parallelism = min(parallelism, capacity);

        let sharding_rate = rate_in_seconds / parallelism;
        let sharding_capacity = capacity / parallelism;
        let remind_rate = rate_in_seconds % parallelism;
        let remind_capacity = capacity % parallelism;

        let mut rates = vec![sharding_rate; parallelism as usize];
        let mut capacities = vec![sharding_capacity; parallelism as usize];

        for i in 0..remind_rate {
            let i = i as usize;
            rates[i] = rates[i] + 1
        }

        for i in 0..remind_capacity {
            let i = i as usize;
            capacities[i] = capacities[i] + 1;
        }

        let mut shards = Vec::with_capacity(parallelism as usize);

        for i in 0..parallelism {
            let i = i as usize;
            shards.push(Arc::new(TokenBucketRateLimiter::new(
                rates[i],
                capacities[i],
            )))
        }

        Self {
            index: AtomicUsize::new(0),
            shards,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_rate_limiter() {
        let limiter = ShardingTokenBucketRateLimiter::new(1, 1);
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(limiter.acquire(), Ok(()));
        assert_eq!(limiter.acquire(), Err(()));
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(limiter.acquire(), Ok(()));
        assert_eq!(limiter.acquire(), Err(()));
    }
}
