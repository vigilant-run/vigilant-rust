# Vigilant Rust SDK

This is the Rust SDK for the Vigilant platform.

## Installation

```bash
cargo add vigilant
```

## Usage (Logger)

```rust
use vigilant::LoggerBuilder;

fn main() {
  let mut logger = LoggerBuilder::new()
    .name("rust-service".to_string())
    .endpoint("localhost:5100".to_string())
    .token("tk_23a93a363620488f".to_string())
    .insecure(true)
    .build();

  logger.info("Hello, world!");

  let _ = logger.shutdown();
}

```
