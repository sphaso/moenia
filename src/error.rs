use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error<E: std::error::Error> {
    #[error("call was rejected because the breaker is open")]
    CircuitOpen,

    #[error("call was rejected because a probe is already running")]
    ProbeInFlight,

    #[error(transparent)]
    Inner(#[from] E),
}
