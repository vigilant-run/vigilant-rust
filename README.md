# Vigilant Rust SDK

This is the Rust SDK for the Vigilant platform.

## Installation

```bash
cargo add vigilant
```

## Usage (with log)

```rust
use log::{debug, info};
use vigilant::EnvLoggerAdapterBuilder;

fn main() {
let adapter = EnvLoggerAdapterBuilder::new()
  .name("rust-app")
  .token("tk_1234567890")
  .build();

log::set_max_level(log::LevelFilter::Debug);
log::set_boxed_logger(Box::new(adapter.clone())).expect("Failed to set logger");

info!("Starting application");
debug!("Debug message");

adapter.shutdown().expect("Failed to shutdown adapter");
}

```

## Usage (with tracing)

```rust
use tracing::info;
use tracing_subscriber::prelude::*;
use vigilant::TracingAdapterBuilder;

fn main() {
let adapter = TracingAdapterBuilder::new()
  .name("rust-app")
  .token("tk_1234567890")
  .build();

tracing_subscriber::registry().with(adapter.clone()).init();

info!("Hello, world!");

adapter.shutdown().expect("Failed to shutdown adapter");
}
```

## Usage (standard logger)

```rust
use vigilant::LoggerBuilder;

fn main() {
let mut logger = LoggerBuilder::new()
  .name("rust-service")
  .token("tk_1234567890")
  .build();

logger.info("Hello, world!");

let _ = logger.shutdown();
}

```
