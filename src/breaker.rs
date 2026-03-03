use std::future::Future;
use std::marker::PhantomData;

use crate::classifier::Classifier;
use crate::config::Config;
use crate::error::Error;
use crate::policy::Policy;
use crate::state::{transition, State};

pub struct CircuitBreaker<E, P, C>
where
    E: std::error::Error,
    P: Policy,
    C: Classifier<E>,
{
    state: State,
    policy: P,
    config: Config,
    classifier: C,
    _phantom: PhantomData<E>,
}

impl<E: std::error::Error, P: Policy, C: Classifier<E>> CircuitBreaker<E, P, C> {
    pub fn new(policy: P, config: Config, classifier: C) -> Self {
        CircuitBreaker {
            policy,
            config,
            classifier,
            state: State::Closed,
            _phantom: PhantomData,
        }
    }

    pub async fn call<F, Fut, T>(&mut self, f: F) -> Result<T, Error<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.pre_call()?;
        let result = f().await;
        self.post_call(&result);

        Ok(result?)
    }

    pub fn call_blocking<F, T>(&mut self, f: F) -> Result<T, Error<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.pre_call()?;
        let result = f();
        self.post_call(&result);

        Ok(result?)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, State::Closed)
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }

    pub fn is_half_open(&self) -> bool {
        matches!(self.state, State::HalfOpen { .. })
    }

    fn pre_call(&mut self) -> Result<(), Error<E>> {
        if let Some(new_state) = transition(
            &self.state,
            &mut self.policy,
            self.config.half_open_probes,
            self.config.open_duration,
        ) {
            self.state = new_state;
        }

        match &self.state {
            State::Open { .. } => return Err(Error::CircuitOpen),
            State::HalfOpen {
                in_flight,
                n_successful_probes,
            } => {
                if *in_flight {
                    return Err(Error::ProbeInFlight);
                } else {
                    self.state = State::HalfOpen {
                        n_successful_probes: *n_successful_probes,
                        in_flight: true,
                    }
                }
            }
            _otherwise => (),
        }
        Ok(())
    }

    fn post_call<T>(&mut self, result: &Result<T, E>) {
        match &result {
            Ok(_) => {
                self.policy.record_success();
                match &self.state {
                    State::HalfOpen {
                        n_successful_probes,
                        ..
                    } => {
                        self.state = State::HalfOpen {
                            n_successful_probes: *n_successful_probes + 1,
                            in_flight: false,
                        }
                    }
                    _otherwise => (),
                }
            }
            Err(e) => {
                if self.classifier.is_failure(e) {
                    if let State::HalfOpen { .. } = self.state {
                        self.state = State::HalfOpen {
                            n_successful_probes: 0,
                            in_flight: false,
                        }
                    }
                    self.policy.record_failure();
                }
            }
        }

        if let Some(new_state) = transition(
            &self.state,
            &mut self.policy,
            self.config.half_open_probes,
            self.config.open_duration,
        ) {
            self.state = new_state;
        }
    }
}
