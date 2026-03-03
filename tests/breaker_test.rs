use moenia::CircuitBreaker;
use moenia::Config;
use moenia::CountBased;
use moenia::{AlwaysFailure, MatchClassifier, NeverFailure};
use std::time::Duration;

#[tokio::test]
async fn never_failure_test() {
    let config = Config::new("never_failure_test");
    let policy = CountBased::new(1);
    let classifier = NeverFailure;

    let mut cb = CircuitBreaker::new(policy, config, classifier);
    let _ = cb
        .call(|| async {
            Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        })
        .await;

    assert!(cb.is_closed());
}

#[tokio::test]
async fn always_failure_test() {
    let config = Config::new("never_failure_test");
    let policy = CountBased::new(1);
    let classifier = AlwaysFailure;

    let mut cb = CircuitBreaker::new(policy, config, classifier);
    let _ = cb
        .call(|| async {
            Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        })
        .await;

    assert!(cb.is_open());
}

#[tokio::test]
async fn half_open_to_closed_transition_test() {
    let config = Config::new("half_open_test")
        .open_duration(Duration::from_millis(1))
        .half_open_probes(1);
    let policy = CountBased::new(1);
    let classifier =
        MatchClassifier::new(|e: &std::io::Error| e.kind() == std::io::ErrorKind::Other);

    let mut cb = CircuitBreaker::new(policy, config, classifier);

    let _ = cb
        .call(|| async {
            Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        })
        .await;
    assert!(cb.is_open());

    std::thread::sleep(Duration::from_millis(10));

    let _ = cb.call(|| async { Ok(1234) }).await;
    assert!(cb.is_closed());
}

#[tokio::test]
async fn half_open_to_open_transition_test() {
    let config = Config::new("half_open_test")
        .open_duration(Duration::from_millis(1))
        .half_open_probes(1);
    let policy = CountBased::new(1);
    let classifier = AlwaysFailure;

    let mut cb = CircuitBreaker::new(policy, config, classifier);

    let _ = cb
        .call(|| async {
            Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        })
        .await;
    assert!(cb.is_open());

    std::thread::sleep(Duration::from_millis(10));

    let _ = cb
        .call(|| async {
            Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
        })
        .await;
    assert!(cb.is_open());
}

#[test]
fn call_blocking_half_open_to_closed_transition_test() {
    let config = Config::new("half_open_test")
        .open_duration(Duration::from_millis(1))
        .half_open_probes(1);
    let policy = CountBased::new(1);
    let classifier =
        MatchClassifier::new(|e: &std::io::Error| e.kind() == std::io::ErrorKind::Other);

    let mut cb = CircuitBreaker::new(policy, config, classifier);

    let _ = cb.call_blocking(|| {
        Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
    });
    assert!(cb.is_open());

    std::thread::sleep(Duration::from_millis(10));

    let _ = cb.call_blocking(|| Ok(1234));
    assert!(cb.is_closed());
}

#[test]
fn call_blocking_half_open_to_open_transition_test() {
    let config = Config::new("half_open_test")
        .open_duration(Duration::from_millis(1))
        .half_open_probes(1);
    let policy = CountBased::new(1);
    let classifier = AlwaysFailure;

    let mut cb = CircuitBreaker::new(policy, config, classifier);

    let _ = cb.call_blocking(|| {
        Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
    });
    assert!(cb.is_open());

    std::thread::sleep(Duration::from_millis(10));

    let _ = cb.call_blocking(|| {
        Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
    });
    assert!(cb.is_open());
}

//  #[tokio::test]
//  async fn half_open_probe_in_flight_test() {
//      let config = Config::new("half_open_test")
//          .open_duration(Duration::from_millis(1))
//          .half_open_probes(1);
//      let policy = CountBased::new(1);
//      let classifier = AlwaysFailure;

//      let mut cb = CircuitBreaker::new(policy, config, classifier);

//      let _ = cb.call(|| async { Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error")) }).await;
//      assert!(cb.is_open());
//      let result = cb.call(|| async { Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "test error")) }).await;

//      assert!(matches!(result, Err(Error::ProbeInFlight)));
//  }
