#![doc = include_str!("../README.md")]
#![feature(doc_cfg)]
#![feature(impl_trait_in_assoc_type)]

mod rate_limit_error;

mod rate_limiter;
pub use rate_limiter::*;

mod rate_limiter_service;
pub use rate_limiter_service::{RateLimiterLayer, RateLimiterService};