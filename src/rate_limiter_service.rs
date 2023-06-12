use crate::rate_limit_error::RateLimitError;
use crate::RateLimiter;

/// A template type that wrappes a [RateLimiter] implmentation as a [volo::Service].
#[derive(Clone)]
pub struct RateLimiterService<S, L> {
    inner: S,
    limiter: std::sync::Arc<L>,
}

#[volo::service]
impl<Cx, Req, S, L> volo::Service<Cx, Req> for RateLimiterService<S, L>
where
    Req: Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    Cx: Send + 'static,
    L: RateLimiter + std::marker::Sync,
    RateLimitError: Into<S::Error>,
{
    async fn call<'cx, 's>(&'s mut self, cx: &'cx mut Cx, req: Req) -> Result<S::Response, S::Error>
    where
        's: 'cx,
    {
        match self.limiter.acquire() {
            Ok(_) => self.inner.call(cx, req).await,
            Err(_) => Err(RateLimitError.into()),
        }
    }
}

/// The implementation of [volo::Layer] for [RateLimiterService].
pub struct RateLimiterLayer<L>(pub L);

impl<S, L> volo::Layer<S> for RateLimiterLayer<L>
where
    L: RateLimiter,
{
    type Service = RateLimiterService<S, L>;

    fn layer(self, inner: S) -> Self::Service {
        RateLimiterService {
            inner: inner,
            limiter: std::sync::Arc::new(self.0),
        }
    }
}