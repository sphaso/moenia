use thiserror::Error;

/// Errors produced by [`crate::CircuitBreaker`].
///
/// Wraps the inner error type `E` with circuit breaker specific variants.
///
/// # Example
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() {
/// use moenia::{CircuitBreaker, Config, CountBased, AlwaysFailure, Error};
///
/// let mut breaker = CircuitBreaker::new(
///     CountBased::new(1),
///     Config::new("my-service"),
///     AlwaysFailure,
/// );
///
/// match breaker.call(|| async {
///     Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
/// }).await {
///     Ok(_) => println!("success"),
///     Err(Error::CircuitOpen) => println!("breaker is open"),
///     Err(Error::ProbeInFlight) => println!("probe already running"),
///     Err(Error::Inner(e)) => println!("service error: {}", e),
/// }
/// # }
/// ```
#[derive(Debug, Error, PartialEq)]
pub enum Error<E: std::error::Error> {
    #[error("call was rejected because the breaker is open")]
    /// The circuit breaker is open — call was rejected without attempting the service.
    CircuitOpen,

    #[error("call was rejected because a probe is already running")]
    /// The circuit breaker is half-open and a probe is already in flight.
    ProbeInFlight,

    #[error(transparent)]
    /// The inner service returned an error.
    Inner(#[from] E),
}
