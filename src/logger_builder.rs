use crate::logger::Logger;

pub struct LoggerBuilder {
    name: String,
    endpoint: String,
    token: String,
    passthrough: bool,
    insecure: bool,
    noop: bool,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            name: "sample-app".to_string(),
            endpoint: "ingress.vigilant.run".to_string(),
            token: "tk_1234567890".to_string(),
            passthrough: false,
            insecure: false,
            noop: false,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub fn token(mut self, token: String) -> Self {
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
