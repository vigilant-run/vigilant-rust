use chrono::Utc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

use crate::types::{Attribute, LogLevel, LogMessage, MessageBatch, MessageType};

pub struct Logger {
    name: String,
    passthrough: bool,
    noop: bool,

    tx: Sender<LogMessage>,
    stop_signal: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
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

        let worker_handle = Some(thread::spawn(move || {
            Self::run_batcher(rx, formatted_endpoint, token, stop_signal_cloned);
        }));

        Logger {
            name,
            passthrough,
            noop,
            tx,
            stop_signal,
            worker_handle,
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

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        self.stop_signal.store(true, Ordering::SeqCst);

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
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

        if let Err(_e) = self.tx.send(log_message) {}

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

        while !stop_signal.load(Ordering::SeqCst) {
            match rx.recv_timeout(batch_interval) {
                Ok(msg) => {
                    buffer.push(msg);
                    if buffer.len() >= max_batch_size {
                        Self::send_batch(&endpoint, &token, &mut buffer);
                    }
                }
                Err(_timeout_or_disconnect) => {
                    if !buffer.is_empty() {
                        Self::send_batch(&endpoint, &token, &mut buffer);
                    }
                }
            }
        }

        if !buffer.is_empty() {
            Self::send_batch(&endpoint, &token, &mut buffer);
        }
    }

    fn send_batch(endpoint: &str, token: &str, buffer: &mut Vec<LogMessage>) {
        if buffer.is_empty() {
            return;
        }
        let current_batch = MessageBatch {
            token: token.to_string(),
            msg_type: MessageType::Logs,
            logs: buffer.drain(..).collect(),
        };

        let client = reqwest::blocking::Client::new();
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
