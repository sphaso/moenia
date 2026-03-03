use std::time::Duration;

/// Configuration for a [`crate::CircuitBreaker`].
///
/// Use the builder methods to customize behavior. All fields have sensible defaults
/// except for `name` which must be provided.
///
/// # Example
///
/// ```rust
/// use moenia::Config;
/// use std::time::Duration;
///
/// let config = Config::new("payments-service")
///     .open_duration(Duration::from_secs(60))
///     .half_open_probes(3);
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    name: String,
    pub open_duration: Duration,
    pub half_open_probes: u32,
}

impl Config {
    /// Creates a new `Config` with the given name and sensible defaults:
    /// - `open_duration`: 30 seconds
    /// - `half_open_probes`: 5
    pub fn new(name: &str) -> Self {
        Config {
            name: name.to_string(),
            open_duration: Duration::from_secs(30),
            half_open_probes: 5,
        }
    }

    /// Sets how long the breaker stays open before allowing a probe call.
    pub fn open_duration(mut self, duration: Duration) -> Self {
        self.open_duration = duration;
        self
    }

    /// Sets how many successful probe calls are required to close the breaker.
    pub fn half_open_probes(mut self, probes: u32) -> Self {
        self.half_open_probes = probes;
        self
    }

    /// Gets the name of the breaker Config
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn open_duration_overrides_default() {
        let duration = Duration::from_secs(31);
        let config = Config::new("test_config").open_duration(duration);

        assert_eq!(config.open_duration, duration);
    }

    #[test]
    fn half_open_probes_overrides_default() {
        let config = Config::new("test_config").half_open_probes(4);

        assert_eq!(config.half_open_probes, 4);
    }
}
