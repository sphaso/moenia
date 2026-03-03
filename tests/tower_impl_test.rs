use moenia::{
    AlwaysFailure, CircuitBreaker, CircuitBreakerLayer, Config, CountBased, Error, NeverFailure,
};
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceExt};

struct MockService {
    should_fail: bool,
}

impl Service<()> for MockService {
    type Response = ();
    type Error = std::io::Error;
    type Future = futures::future::BoxFuture<'static, Result<(), std::io::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        if self.should_fail {
            Box::pin(async { Err(std::io::Error::new(std::io::ErrorKind::Other, "mock error")) })
        } else {
            Box::pin(async { Ok(()) })
        }
    }
}

#[tokio::test]
async fn tower_call_succeeds() {
    let config = Config::new("never_failure_test");
    let policy = CountBased::new(1);
    let classifier = NeverFailure;

    let cb: Arc<CircuitBreaker<std::io::Error, CountBased, NeverFailure>> =
        Arc::new(CircuitBreaker::new(policy, config, classifier));
    let layer = CircuitBreakerLayer::new(Arc::clone(&cb));
    let mock_service = MockService { should_fail: false };
    let mut service = layer.layer(mock_service);
    service.ready().await.unwrap();
    let result = service.call(()).await;

    assert!(result.is_ok());
    assert!(cb.is_closed());
}

#[tokio::test]
async fn tower_call_fails() {
    let config = Config::new("never_failure_test");
    let policy = CountBased::new(1);
    let classifier = AlwaysFailure;

    let cb: Arc<CircuitBreaker<std::io::Error, CountBased, AlwaysFailure>> =
        Arc::new(CircuitBreaker::new(policy, config, classifier));
    let layer = CircuitBreakerLayer::new(Arc::clone(&cb));
    let mock_service = MockService { should_fail: true };
    let mut service = layer.layer(mock_service);
    service.ready().await.unwrap();
    let result = service.call(()).await;

    assert!(result.is_err());
    assert!(cb.is_open());
}

#[tokio::test]
async fn tower_call_fails_circuit_open() {
    let config = Config::new("never_failure_test");
    let policy = CountBased::new(1);
    let classifier = AlwaysFailure;

    let cb: Arc<CircuitBreaker<std::io::Error, CountBased, AlwaysFailure>> =
        Arc::new(CircuitBreaker::new(policy, config, classifier));
    let layer = CircuitBreakerLayer::new(Arc::clone(&cb));
    let mock_service = MockService { should_fail: true };
    let mut service = layer.layer(mock_service);
    service.ready().await.unwrap();
    let _ = service.call(()).await;
    let result = service.call(()).await;
    assert!(matches!(result, Err(Error::CircuitOpen)));
}
