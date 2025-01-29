use crate::types::LogLevel;
use crate::{logger::Logger as VigilantLogger, EnvLoggerAdapterBuilder};
use env_logger::{Builder as EnvLoggerBuilder, Logger as EnvLogger};
use log::{Level, Log, Metadata, Record};
use std::sync::Arc;
use std::sync::Mutex;

pub struct EnvLoggerAdapter {
    env_logger: EnvLogger,
    vigilant_logger: Arc<Mutex<VigilantLogger>>,
}

impl EnvLoggerAdapter {
    pub fn new(vigilant_logger: VigilantLogger) -> Result<Arc<Self>, log::SetLoggerError> {
        let env_logger = EnvLoggerBuilder::from_default_env().build();
        let adapter = Arc::new(Self {
            env_logger,
            vigilant_logger: Arc::new(Mutex::new(vigilant_logger)),
        });

        log::set_max_level(log::LevelFilter::Debug);
        log::set_boxed_logger(Box::new(Arc::clone(&adapter)))?;
        Ok(adapter)
    }

    pub fn builder<'a>() -> EnvLoggerAdapterBuilder<'a> {
        EnvLoggerAdapterBuilder::new()
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        if let Ok(mut logger) = self.vigilant_logger.lock() {
            logger.shutdown()?;
        }
        Ok(())
    }
}

impl Log for EnvLoggerAdapter {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.env_logger.log(record);

        if !self.enabled(record.metadata()) {
            return;
        }

        if let Ok(logger) = self.vigilant_logger.lock() {
            let level = match record.level() {
                Level::Error => LogLevel::ERROR,
                Level::Warn => LogLevel::WARNING,
                Level::Info => LogLevel::INFO,
                Level::Debug | Level::Trace => LogLevel::DEBUG,
            };

            let message = record.args().to_string();
            match level {
                LogLevel::ERROR => logger.error(&message),
                LogLevel::WARNING => logger.warn(&message),
                LogLevel::INFO => logger.info(&message),
                LogLevel::DEBUG => logger.debug(&message),
            }
        }
    }

    fn flush(&self) {}
}
