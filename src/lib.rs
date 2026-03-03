mod breaker;
mod classifier;
mod config;
mod error;
mod policy;
mod state;
#[cfg(feature = "tower")] mod tower_impl;

pub use breaker::CircuitBreaker;
pub use classifier::{AlwaysFailure, Classifier, MatchClassifier, NeverFailure};
pub use config::Config;
pub use error::Error;
pub use policy::{CountBased, Policy, SlidingWindow};
#[cfg(feature = "tower")] pub use tower_impl::{CircuitBreakerLayer, CircuitBreakerService};
