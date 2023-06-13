/// The error type returned by [RateLimiterService](crate::RateLimiterService).
///
/// The [volo_thrift] using [volo_thrift::AnyhowError] as error type and it has a default way of converting from `std::error::Error`
/// so it's not necessary (and will cause conflict) for [RateLimitError] to implement `Into<volo_thrift::AnyhowError>`.
///
/// This also leads to a potential problem, when the request rejected by this limiter, the error type is always "ApplicationError" and
/// error kind is always "Unknown", making it difficult for clients and monitor components to identify the failure caused by the limiter.
#[derive(Debug)]
pub(crate) struct RateLimitError;

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "rate limited")
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(feature = "volo-grpc")]
impl Into<volo_grpc::Status> for RateLimitError {
    fn into(self) -> volo_grpc::Status {
        volo_grpc::Status::resource_exhausted(self.to_string())
    }
}
