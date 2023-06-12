/// A bucket rate limiter implementation, using a dedicated [tokio] task as token producer.
///
/// This rate limiter implementation requires the server using [tokio] as its runtime.
#[doc(cfg(feature = "tokio"))]
#[derive(Clone)]
pub struct TokioBucketRateLimiter {
    status: std::sync::Arc<TokioBucketRateLimiterStatus>,

    // wrapped in `Arc<Mutex<...>>` to satisfy `Clone` and `Send` requirements.
    handle: std::sync::Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[doc(cfg(feature = "tokio"))]
struct TokioBucketRateLimiterStatus {
    duration: tokio::time::Duration,
    quota: i64,

    tokens: std::sync::atomic::AtomicI64,

    notify: tokio::sync::Notify,
}

#[doc(cfg(feature = "tokio"))]
impl crate::RateLimiter for TokioBucketRateLimiter {
    fn acquire(&self) -> Result<(), ()> {
        match self
            .status
            .tokens
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
        {
            1.. => Ok(()),
            _ => {
                self.status
                    .tokens
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(())
            }
        }
    }
}

#[doc(cfg(feature = "tokio"))]
impl Drop for TokioBucketRateLimiter {
    fn drop(&mut self) {
        self.status.notify.notify_one();

        // XXX: block on async funtion in sync function
        futures::executor::block_on(self.handle.lock().unwrap().take().unwrap())
            .expect("joining task panicked");
    }
}

#[doc(cfg(feature = "tokio"))]
impl TokioBucketRateLimiter {
    pub fn new(duration: impl Into<tokio::time::Duration>, quota: u64) -> Self {
        let quota: i64 = quota.try_into().expect("limit quota out of range");

        let status = std::sync::Arc::new(TokioBucketRateLimiterStatus {
            duration: duration.into(),
            quota: quota,
            tokens: std::sync::atomic::AtomicI64::new(quota),
            notify: tokio::sync::Notify::new(),
        });

        let status_clone = status.clone();
        let handle = tokio::spawn(async move {
            TokioBucketRateLimiter::proc(status_clone).await;
        });

        Self {
            status,
            handle: std::sync::Arc::new(std::sync::Mutex::new(Some(handle))),
        }
    }

    async fn proc(status: std::sync::Arc<TokioBucketRateLimiterStatus>) {
        let mut instant = tokio::time::Instant::now();
        loop {
            instant += status.duration;

            tokio::select! {
                _ = status.notify.notified() => {
                    break;
                },
                _ = tokio::time::sleep_until(instant) => {
                    status.tokens.store(status.quota, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }
    }
}