use std::future::Future;
use std::marker::PhantomData;
use std::sync::Mutex;

use crate::classifier::Classifier;
use crate::config::Config;
use crate::error::Error;
use crate::policy::Policy;
use crate::state::{transition, State};

struct BreakerState<P: Policy> {
    state: State,
    policy: P,
}

pub struct CircuitBreaker<E, P, C>
where
    E: std::error::Error,
    P: Policy,
    C: Classifier<E>,
{
    inner: Mutex<BreakerState<P>>,
    config: Config,
    classifier: C,
    _phantom: PhantomData<E>,
}

impl<E: std::error::Error, P: Policy, C: Classifier<E>> CircuitBreaker<E, P, C> {
    pub fn new(policy: P, config: Config, classifier: C) -> Self {
        CircuitBreaker {
            inner: Mutex::new(BreakerState { policy, state: State::Closed }),
            config,
            classifier,
            _phantom: PhantomData,
        }
    }

    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, Error<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.pre_call()?;
        let result = f().await;
        self.post_call(&result);

        Ok(result?)
    }

    pub fn call_blocking<F, T>(&self, f: F) -> Result<T, Error<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.pre_call()?;
        let result = f();
        self.post_call(&result);

        Ok(result?)
    }

    pub fn is_closed(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        matches!(inner.state, State::Closed)
    }

    pub fn is_open(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        matches!(inner.state, State::Open { .. })
    }

    pub fn is_half_open(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        matches!(inner.state, State::HalfOpen { .. })
    }

    #[cfg(test)]
    pub fn set_state(&self, state: State) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = state;
    }

    fn pre_call(&self) -> Result<(), Error<E>> {
        let mut inner = self.inner.lock().unwrap();

        let state_snapshot = inner.state.clone();
        if let Some(new_state) = transition(
            &state_snapshot,
            &mut inner.policy,
            self.config.half_open_probes,
            self.config.open_duration,
        ) {
            inner.state = new_state;
        }

        match &inner.state {
            State::Open { .. } => return Err(Error::CircuitOpen),
            State::HalfOpen {
                in_flight,
                n_successful_probes,
            } => {
                if *in_flight {
                    return Err(Error::ProbeInFlight);
                } else {
                    inner.state = State::HalfOpen {
                        n_successful_probes: *n_successful_probes,
                        in_flight: true,
                    }
                }
            }
            _otherwise => (),
        }
        Ok(())
    }

    fn post_call<T>(&self, result: &Result<T, E>) {
        let mut inner = self.inner.lock().unwrap();

        match &result {
            Ok(_) => {
                inner.policy.record_success();
                match &inner.state {
                    State::HalfOpen {
                        n_successful_probes,
                        ..
                    } => {
                        inner.state = State::HalfOpen {
                            n_successful_probes: *n_successful_probes + 1,
                            in_flight: false,
                        }
                    }
                    _otherwise => (),
                }
            }
            Err(e) => {
                if self.classifier.is_failure(e) {
                    if let State::HalfOpen { .. } = inner.state {
                        inner.state = State::HalfOpen {
                            n_successful_probes: 0,
                            in_flight: false,
                        }
                    }
                    inner.policy.record_failure();
                }
            }
        }

        let state_snapshot = inner.state.clone();
        if let Some(new_state) = transition(
            &state_snapshot,
            &mut inner.policy,
            self.config.half_open_probes,
            self.config.open_duration,
        ) {
            inner.state = new_state;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::policy::CountBased;
    use crate::classifier::AlwaysFailure;
    use crate::config::Config;
    use std::time::Duration;

    #[tokio::test]
    async fn probe_in_flight_returns_error() {
        let config = Config::new("half_open_test")
            .open_duration(Duration::from_millis(1))
            .half_open_probes(1);
        let policy = CountBased::new(1);
        let classifier = AlwaysFailure;

        let cb : CircuitBreaker<std::io::Error, CountBased, AlwaysFailure> = CircuitBreaker::new(policy, config, classifier);
        cb.set_state(State::HalfOpen { n_successful_probes: 0, in_flight: true });

        let result = cb.call(|| async { Ok::<(), _>(()) }).await;
        assert!(matches!(result, Err(Error::ProbeInFlight)));
    }
}
