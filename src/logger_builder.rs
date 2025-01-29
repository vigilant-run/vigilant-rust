use crate::logger::Logger;

pub struct LoggerBuilder<'a> {
    name: &'a str,
    endpoint: &'a str,
    token: &'a str,
    passthrough: bool,
    insecure: bool,
    noop: bool,
}

impl<'a> LoggerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: "sample-app",
            endpoint: "ingress.vigilant.run",
            token: "tk_1234567890",
            passthrough: false,
            insecure: false,
            noop: false,
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

    pub fn build(self) -> Logger {
        Logger::new(
            self.name,
            self.endpoint,
            self.token,
            self.passthrough,
            self.insecure,
            self.noop,
        )
    }
}
