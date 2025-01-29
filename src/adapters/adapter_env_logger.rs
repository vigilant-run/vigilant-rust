use crate::types::LogLevel;
use crate::{logger::Logger as VigilantLogger, EnvLoggerAdapterBuilder};
use env_logger::{Builder as EnvLoggerBuilder, Logger as EnvLogger};
use log::{Level, Log, Metadata, Record};
use std::sync::Arc;

#[derive(Clone)]
pub struct EnvLoggerAdapter {
    inner: Arc<EnvLoggerAdapterInner>,
}

struct EnvLoggerAdapterInner {
    env_logger: Arc<EnvLogger>,
    vigilant_logger: VigilantLogger,
}

impl EnvLoggerAdapter {
    pub fn new(vigilant_logger: VigilantLogger) -> Self {
        let env_logger = EnvLoggerBuilder::from_default_env().build();
        Self {
            inner: Arc::new(EnvLoggerAdapterInner {
                env_logger: Arc::new(env_logger),
                vigilant_logger,
            }),
        }
    }

    pub fn builder<'a>() -> EnvLoggerAdapterBuilder<'a> {
        EnvLoggerAdapterBuilder::new()
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        self.inner.vigilant_logger.shutdown()
    }
}

impl Log for EnvLoggerAdapter {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.env_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.inner.env_logger.log(record);

        if !self.enabled(record.metadata()) {
            return;
        }

        let level = match record.level() {
            Level::Error => LogLevel::ERROR,
            Level::Warn => LogLevel::WARNING,
            Level::Info => LogLevel::INFO,
            Level::Debug | Level::Trace => LogLevel::DEBUG,
        };

        let message = record.args().to_string();
        match level {
            LogLevel::ERROR => self.inner.vigilant_logger.error(&message),
            LogLevel::WARNING => self.inner.vigilant_logger.warn(&message),
            LogLevel::INFO => self.inner.vigilant_logger.info(&message),
            LogLevel::DEBUG => self.inner.vigilant_logger.debug(&message),
        }
    }

    fn flush(&self) {}
}
