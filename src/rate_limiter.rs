/// The interface definition of a rate limiter.
pub trait RateLimiter: Send {
    /// Try to acquire a request quota.
    ///
    /// If the request is determined to be passed, the method returns `Ok(())`, otherwise returns `Err(())`
    fn acquire(&self) -> Result<(), ()>;
}

mod inaccurate_bucket_rate_limiter;
pub use inaccurate_bucket_rate_limiter::*;

mod threading_bucket_rate_limiter;
pub use threading_bucket_rate_limiter::*;

mod token_bucket_rate_limiter;
pub use token_bucket_rate_limiter::*;

#[cfg(feature = "tokio")]
mod tokio_bucket_rate_limiter;
#[cfg(feature = "tokio")]
pub use tokio_bucket_rate_limiter::*;
