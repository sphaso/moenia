use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct CountBased {
    failures: u32,
    threshold: u32,
}

impl CountBased {
    pub fn new(threshold: u32) -> Self {
        CountBased {
            threshold,
            failures: 0,
        }
    }
}

pub struct SlidingWindow {
    failures: VecDeque<Instant>,
    window: Duration,
    threshold: u32,
}

impl SlidingWindow {
    pub fn new(threshold: u32, window: Duration) -> Self {
        SlidingWindow {
            threshold,
            window,
            failures: VecDeque::new(),
        }
    }
}

pub trait Policy {
    fn record_success(&mut self);
    fn record_failure(&mut self);
    fn should_open(&self) -> bool;
    fn reset(&mut self);
}

impl Policy for CountBased {
    fn record_success(&mut self) {
        self.failures = self.failures.saturating_sub(1);
    }

    fn record_failure(&mut self) {
        self.failures += 1;
    }

    fn should_open(&self) -> bool {
        self.failures >= self.threshold
    }

    fn reset(&mut self) {
        self.failures = 0;
    }
}

impl Policy for SlidingWindow {
    fn record_success(&mut self) {}

    fn record_failure(&mut self) {
        let now = Instant::now();
        self.failures.retain(|f| now - *f < self.window);
        self.failures.push_front(now);
    }

    fn should_open(&self) -> bool {
        self.failures.len() >= self.threshold as usize
    }

    fn reset(&mut self) {
        self.failures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_based_opens_at_threshold() {
        let mut policy = CountBased::new(5);
        policy.record_failure();
        policy.record_failure();
        policy.record_failure();
        policy.record_failure();
        assert!(!policy.should_open());
        policy.record_failure();
        assert!(policy.should_open());
    }

    #[test]
    fn sliding_window_opens_at_threshold() {
        let mut policy = SlidingWindow::new(5, Duration::from_secs(30));
        policy.record_failure();
        policy.record_failure();
        policy.record_failure();
        policy.record_failure();
        assert!(!policy.should_open());
        policy.record_failure();
        assert!(policy.should_open());
    }

    #[test]
    fn sliding_window_eviction() {
        let mut policy = SlidingWindow::new(2, Duration::from_secs(1));
        policy.record_failure();
        std::thread::sleep(Duration::from_secs(1));
        policy.record_failure();
        assert!(!policy.should_open());
    }
}
