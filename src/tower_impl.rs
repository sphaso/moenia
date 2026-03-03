use futures::future::BoxFuture;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;

use crate::breaker::CircuitBreaker;
use crate::classifier::Classifier;
use crate::policy::Policy;

/// A [`tower::Layer`] that wraps a service with circuit breaker protection.
///
/// # Example
///
/// ```rust,no_run
/// use moenia::{CircuitBreaker, CircuitBreakerLayer, Config, CountBased, AlwaysFailure};
/// use tower::ServiceBuilder;
/// use std::sync::Arc;
///
/// let breaker = Arc::new(CircuitBreaker::new(
///     CountBased::new(5),
///     Config::new("payments-service"),
///     AlwaysFailure,
/// ));
///
/// let layer = CircuitBreakerLayer::new(breaker);
/// ```
#[derive(Clone)]
pub struct CircuitBreakerLayer<E, P, C>
where
    E: std::error::Error,
    P: Policy,
    C: Classifier<E>,
{
    breaker: Arc<CircuitBreaker<E, P, C>>,
}

/// A [`tower::Service`] that wraps an inner service with circuit breaker protection.
///
/// Created by [`CircuitBreakerLayer`] — you rarely need to construct this directly.
#[derive(Clone)]
pub struct CircuitBreakerService<E, P, C, S>
where
    E: std::error::Error,
    P: Policy,
    C: Classifier<E>,
{
    breaker: Arc<CircuitBreaker<E, P, C>>,
    inner: S,
}

impl<E: std::error::Error, P: Policy, C: Classifier<E>> CircuitBreakerLayer<E, P, C> {
    pub fn new(breaker: Arc<CircuitBreaker<E, P, C>>) -> Self {
        CircuitBreakerLayer { breaker }
    }
}

impl<E, P, C, S> Layer<S> for CircuitBreakerLayer<E, P, C>
where
    E: std::error::Error,
    P: Policy,
    C: Classifier<E>,
{
    type Service = CircuitBreakerService<E, P, C, S>;

    fn layer(&self, inner: S) -> Self::Service {
        CircuitBreakerService {
            inner,
            breaker: Arc::clone(&self.breaker),
        }
    }
}

impl<E, P, C, S, Req> Service<Req> for CircuitBreakerService<E, P, C, S>
where
    E: std::error::Error + Send + Sync + 'static,
    P: Policy + Send + Sync + 'static,
    C: Classifier<E> + Send + Sync + 'static,
    S: Service<Req>,
    S::Error: Into<E>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = crate::Error<E>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| crate::Error::Inner(e.into()))
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let breaker = Arc::clone(&self.breaker);
        let inner_future = self.inner.call(req);

        Box::pin(async move {
            breaker
                .call(|| async move { inner_future.await.map_err(Into::into) })
                .await
        })
    }
}
