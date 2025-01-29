use chrono::Utc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use crate::types::{Attribute, LogLevel, LogMessage, MessageBatch, MessageType};

pub struct Logger {
    name: String,
    passthrough: bool,
    noop: bool,
    inner: Arc<LoggerInner>,
}

struct LoggerInner {
    tx: Sender<LogMessage>,
    stop_signal: Arc<AtomicBool>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            passthrough: self.passthrough,
            noop: self.noop,
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Logger {
    pub fn new<S: Into<String>>(
        name: S,
        endpoint: S,
        token: S,
        passthrough: bool,
        insecure: bool,
        noop: bool,
    ) -> Self {
        let name = name.into();
        let endpoint = endpoint.into();
        let token = token.into();

        let formatted_endpoint = if insecure {
            format!("http://{}/api/message", endpoint)
        } else {
            format!("https://{}/api/message", endpoint)
        };

        let (tx, rx) = channel::<LogMessage>();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_cloned = Arc::clone(&stop_signal);

        let worker_handle = thread::spawn(move || {
            Self::run_batcher(rx, formatted_endpoint, token, stop_signal_cloned);
        });

        Logger {
            name,
            passthrough,
            noop,
            inner: Arc::new(LoggerInner {
                tx,
                stop_signal,
                worker_handle: Mutex::new(Some(worker_handle)),
            }),
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::DEBUG, message, None, Vec::new());
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::WARNING, message, None, Vec::new());
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::INFO, message, None, Vec::new());
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::ERROR, message, None, Vec::new());
    }

    pub fn debug_with_attrs(&self, message: &str, attrs: impl IntoIterator<Item = Attribute>) {
        self.log(LogLevel::DEBUG, message, None, attrs);
    }

    pub fn warn_with_attrs(&self, message: &str, attrs: impl IntoIterator<Item = Attribute>) {
        self.log(LogLevel::WARNING, message, None, attrs);
    }

    pub fn info_with_attrs(&self, message: &str, attrs: impl IntoIterator<Item = Attribute>) {
        self.log(LogLevel::INFO, message, None, attrs);
    }

    pub fn error_with_attrs(&self, message: &str, attrs: impl IntoIterator<Item = Attribute>) {
        self.log(LogLevel::ERROR, message, None, attrs);
    }

    pub fn shutdown(&self) -> std::io::Result<()> {
        self.inner.stop_signal.store(true, Ordering::SeqCst);
        if let Ok(mut handle) = self.inner.worker_handle.lock() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
        Ok(())
    }

    fn log(
        &self,
        level: LogLevel,
        message: &str,
        err: Option<&dyn std::error::Error>,
        attrs: impl IntoIterator<Item = Attribute>,
    ) {
        if self.noop {
            return;
        }

        let mut map = std::collections::HashMap::new();
        map.insert("service.name".to_string(), self.name.clone());

        for attr in attrs {
            map.insert(attr.key, attr.value);
        }

        if let Some(e) = err {
            map.insert("error".to_string(), e.to_string());
        }

        let log_message = LogMessage {
            timestamp: current_timestamp_rfc3339(),
            body: message.to_string(),
            level,
            attributes: map,
        };

        if let Err(_e) = self.inner.tx.send(log_message) {}

        self.log_passthrough(level, message, err);
    }

    fn log_passthrough(&self, level: LogLevel, message: &str, err: Option<&dyn std::error::Error>) {
        if !self.passthrough {
            return;
        }

        if let Some(e) = err {
            println!("[{:?}] {} error=\"{}\"", level, message, e);
        } else {
            println!("[{:?}] {}", level, message);
        }
    }

    fn run_batcher(
        rx: Receiver<LogMessage>,
        endpoint: String,
        token: String,
        stop_signal: Arc<AtomicBool>,
    ) {
        let max_batch_size = 100;
        let batch_interval = Duration::from_millis(100);
        let mut buffer = Vec::with_capacity(max_batch_size);
        let client = reqwest::blocking::Client::new();

        loop {
            match rx.recv_timeout(batch_interval) {
                Ok(msg) => {
                    buffer.push(msg);
                    if buffer.len() >= max_batch_size {
                        Self::send_batch(&client, &endpoint, &token, &mut buffer);
                    }
                }
                Err(_timeout_or_disconnect) => {
                    if !buffer.is_empty() {
                        Self::send_batch(&client, &endpoint, &token, &mut buffer);
                    }
                    if stop_signal.load(Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }

        if !buffer.is_empty() {
            Self::send_batch(&client, &endpoint, &token, &mut buffer);
        }
    }

    fn send_batch(
        client: &reqwest::blocking::Client,
        endpoint: &str,
        token: &str,
        buffer: &mut Vec<LogMessage>,
    ) {
        if buffer.is_empty() {
            return;
        }
        let current_batch = MessageBatch {
            token: token.to_string(),
            msg_type: MessageType::Logs,
            logs: buffer.drain(..).collect(),
        };

        if let Err(e) = client
            .post(endpoint)
            .json(&current_batch)
            .header("Content-Type", "application/json")
            .send()
        {
            eprintln!("Failed to send log batch: {}", e);
        }
    }
}

fn current_timestamp_rfc3339() -> String {
    Utc::now().to_rfc3339()
}
