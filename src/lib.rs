//! # moenia
//!
//! *Defensive walls for your services — a circuit breaker library for Rust.*
//!
//! `moenia` protects your application from cascading failures by wrapping calls
//! to external services and automatically stopping requests when a service is
//! struggling, giving it time to recover.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use moenia::{CircuitBreaker, Config, CountBased, AlwaysFailure};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let breaker = CircuitBreaker::new(
//!     CountBased::new(5),
//!     Config::new("payments-service")
//!         .open_duration(Duration::from_secs(30))
//!         .half_open_probes(2),
//!     AlwaysFailure,
//! );
//!
//! let result = breaker.call(|| async {
//!     // your service call here
//!     Ok::<(), std::io::Error>(())
//! }).await;
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `tower` | Tower middleware integration for axum, tonic, hyper | disabled |
//! | `otel` | OpenTelemetry metrics instrumentation | disabled |
//! | `serde` | Serde support for config serialization | disabled |
//!
//! ## How It Works
//!
//! `moenia` implements the classic circuit breaker pattern with three states:
//!
//! - **Closed** — normal operation, calls pass through, failures are counted
//! - **Open** — too many failures, calls are rejected immediately
//! - **Half-Open** — after a timeout, a probe call is allowed through to test recovery
//!
//! ## Key Types
//!
//! - [`CircuitBreaker`] — the main circuit breaker, wraps your service calls
//! - [`Config`] — builder for circuit breaker configuration
//! - [`CountBased`] — trips after N absolute failures
//! - [`SlidingWindow`] — trips after a failure rate within a time window
//! - [`MatchClassifier`] — classify errors with a closure

mod breaker;
mod classifier;
mod config;
mod error;
mod policy;
mod state;
#[cfg(feature = "tower")]
mod tower_impl;

pub use breaker::CircuitBreaker;
pub use classifier::{AlwaysFailure, Classifier, MatchClassifier, NeverFailure};
pub use config::Config;
pub use error::Error;
pub use policy::{CountBased, Policy, SlidingWindow};
#[cfg(feature = "tower")]
pub use tower_impl::{CircuitBreakerLayer, CircuitBreakerService};
