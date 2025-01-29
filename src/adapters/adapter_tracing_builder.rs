use crate::{LoggerBuilder, TracingAdapter};
use tracing::level_filters::LevelFilter;

pub struct TracingAdapterBuilder<'a> {
    name: &'a str,
    endpoint: &'a str,
    token: &'a str,
    passthrough: bool,
    insecure: bool,
    noop: bool,
    level_filter: LevelFilter,
}

impl<'a> TracingAdapterBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: "sample-app",
            endpoint: "ingress.vigilant.run",
            token: "tk_1234567890",
            passthrough: false,
            insecure: false,
            noop: false,
            level_filter: LevelFilter::INFO,
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn endpoint(mut self, endpoint: &'a str) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub fn token(mut self, token: &'a str) -> Self {
        self.token = token;
        self
    }

    pub fn passthrough(mut self, enabled: bool) -> Self {
        self.passthrough = enabled;
        self
    }

    pub fn insecure(mut self, enabled: bool) -> Self {
        self.insecure = enabled;
        self
    }

    pub fn noop(mut self, enabled: bool) -> Self {
        self.noop = enabled;
        self
    }

    pub fn level_filter(mut self, level_filter: LevelFilter) -> Self {
        self.level_filter = level_filter;
        self
    }

    pub fn build(self) -> TracingAdapter {
        let vigilant_logger = LoggerBuilder::new()
            .name(self.name)
            .endpoint(self.endpoint)
            .token(self.token)
            .passthrough(self.passthrough)
            .insecure(self.insecure)
            .noop(self.noop)
            .build();

        TracingAdapter::new(vigilant_logger, self.level_filter)
    }
}
