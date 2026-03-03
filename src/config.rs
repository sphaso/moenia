use std::time::Duration;

pub struct Config {
    name: String,
    pub open_duration: Duration,
    pub half_open_probes: u32,
}

impl Config {
    pub fn new(name: &str) -> Self {
        Config {
            name: name.to_string(),
            open_duration: Duration::from_secs(30),
            half_open_probes: 5,
        }
    }

    pub fn open_duration(mut self, duration: Duration) -> Self {
        self.open_duration = duration;
        self
    }

    pub fn half_open_probes(mut self, probes: u32) -> Self {
        self.half_open_probes = probes;
        self
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
