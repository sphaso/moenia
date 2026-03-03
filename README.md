# moenia
[![Build Status](https://github.com/sphaso/moenia/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sphaso/moenia/actions/workflows/ci.yml)
> *Moenia* — Latin for "defensive walls". A circuit breaker library for Rust.

`moenia` protects your application from cascading failures by wrapping calls to external services and automatically stopping requests when a service is struggling, giving it time to recover.

## Features

- **Async and sync** — `call` for async, `call_blocking` for sync
- **Two failure policies** — count-based or sliding window
- **Flexible error classification** — define what counts as a failure
- **Tower integration** — plugs into axum, tonic, and hyper via the `tower` feature
- **OpenTelemetry native** — metrics and state transition tracking via the `otel` feature
- **Zero required dependencies** — everything is opt-in via feature flags

## Quick Start
```toml
[dependencies]
moenia = "0.1"
```
```rust
use moenia::{CircuitBreaker, Config, CountBased, MatchClassifier};
use std::time::Duration;

let breaker = CircuitBreaker::new(
    CountBased::new(5),
    Config::new("payments-service")
        .open_duration(Duration::from_secs(30))
        .half_open_probes(2),
    MatchClassifier::new(|e: &reqwest::Error| e.is_timeout() || e.is_connect()),
);

let result = breaker.call(|| async {
    client.get("https://payments-api/charge").send().await
}).await;

match result {
    Ok(response) => // handle response,
    Err(moenia::Error::CircuitOpen) => // fallback,
    Err(moenia::Error::Inner(e)) => // handle error,
    _ => ()
}
```

## Tower Integration
```rust
use moenia::{CircuitBreaker, CircuitBreakerLayer, Config, CountBased, AlwaysFailure};
use tower::ServiceBuilder;
use std::sync::Arc;

let breaker = Arc::new(CircuitBreaker::new(
    CountBased::new(5),
    Config::new("payments-service"),
    AlwaysFailure,
));

let service = ServiceBuilder::new()
    .layer(CircuitBreakerLayer::new(breaker))
    .service(inner_service);
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `tower` | Tower middleware integration | disabled |
| `otel` | OpenTelemetry metrics instrumentation | disabled |
| `serde` | Serde support for `Config` serialization | disabled |

## How It Works
```
Closed ──(threshold exceeded)──▶ Open
  ▲                                │
  │                          (timeout elapsed)
  │                                ▼
  └──(probe succeeds)───── HalfOpen
```

- **Closed** — normal operation, failures are counted
- **Open** — calls rejected immediately, service gets time to recover
- **Half-Open** — probe calls test recovery, success closes the breaker

## License

Licensed under the Unlicense.
