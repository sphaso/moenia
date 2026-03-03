use crate::policy::Policy;
use std::time::{Duration, Instant};

pub enum State {
    Open {
        until: Instant,
    },
    HalfOpen {
        n_successful_probes: u32,
        in_flight: bool,
    },
    Closed,
}

pub fn transition(
    state: &State,
    policy: &mut impl Policy,
    half_open_probes: u32,
    open_duration: Duration,
) -> Option<State> {
    match state {
        State::Open { until } => {
            if Instant::now() >= *until {
                Some(State::HalfOpen {
                    n_successful_probes: 0,
                    in_flight: false,
                })
            } else {
                None
            }
        }
        State::HalfOpen {
            n_successful_probes,
            in_flight,
        } => {
            if *n_successful_probes >= half_open_probes {
                policy.reset();
                Some(State::Closed)
            } else if !in_flight {
                Some(State::Open {
                    until: Instant::now() + open_duration,
                })
            } else {
                None
            }
        }
        State::Closed => {
            if policy.should_open() {
                Some(State::Open {
                    until: Instant::now() + open_duration,
                })
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CountBased;

    #[test]
    fn closed_to_closed_should_open_false() {
        let mut policy = CountBased::new(5);
        let state = State::Closed;

        let result = transition(&state, &mut policy, 1, Duration::from_secs(5));
        assert!(result.is_none());
    }

    #[test]
    fn closed_to_open_should_open_true() {
        let mut policy = CountBased::new(1);
        let duration = Duration::from_secs(5);
        let state = State::Closed;

        policy.record_failure();
        let result = transition(&state, &mut policy, 1, duration);

        assert!(matches!(result, Some(State::Open { .. })));
    }

    #[test]
    fn open_to_open_time_not_elapsed() {
        let mut policy = CountBased::new(5);
        let duration = Duration::from_secs(5);
        let state = State::Open {
            until: Instant::now() + duration,
        };

        let result = transition(&state, &mut policy, 1, duration);
        assert!(result.is_none());
    }

    #[test]
    fn open_to_halfopen_time_elapsed() {
        let mut policy = CountBased::new(5);
        let duration = Duration::from_secs(1);
        let state = State::Open {
            until: Instant::now() + duration,
        };

        std::thread::sleep(Duration::from_secs(1));

        let result = transition(&state, &mut policy, 1, duration);
        assert!(matches!(result, Some(State::HalfOpen { .. })));
    }

    #[test]
    fn halfopen_to_closed_probe_success_threshold() {
        let mut policy = CountBased::new(5);
        let duration = Duration::from_secs(1);
        let state = State::HalfOpen {
            n_successful_probes: 1,
            in_flight: false,
        };

        let result = transition(&state, &mut policy, 1, duration);
        assert!(matches!(result, Some(State::Closed)));
    }

    #[test]
    fn halfopen_to_halfopen_probe_in_flight() {
        let mut policy = CountBased::new(5);
        let duration = Duration::from_secs(1);
        let state = State::HalfOpen {
            n_successful_probes: 0,
            in_flight: true,
        };

        let result = transition(&state, &mut policy, 1, duration);
        assert!(result.is_none());
    }

    #[test]
    fn halfopen_to_open_probe_failed() {
        let mut policy = CountBased::new(5);
        let duration = Duration::from_secs(1);
        let state = State::HalfOpen {
            n_successful_probes: 0,
            in_flight: false,
        };

        let result = transition(&state, &mut policy, 1, duration);
        assert!(matches!(result, Some(State::Open { .. })));
    }
}
