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
    .endpoint("ingress.vigilant.run".to_string())
    .token("tk_1234567890".to_string())
    .build();

  logger.info("Hello, world!");

  let _ = logger.shutdown();
}

```
