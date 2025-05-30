# New York Calculate Core

A Rust library for financial calculations and trading operations.

## Overview

This library provides core functionality for financial calculations and trading operations, including order management, command processing, and statistical analysis.

## Features

- Order management system
- Command processing
- Statistical calculations
- Agent-based operations
- Candle data handling
- Activation/deactivation logic

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
new_york_calculate_core = "0.0.66"
```

## Usage

```rust
use new_york_calculate_core::{Order, Command, Result};

// Create a new order
let order = Order::new(/* parameters */);

// Process commands
let command = Command::new(/* parameters */);
let result = command.execute();
```

## Development

3. Run `cargo build` to build the project
4. Run `cargo test` to run tests

## License

MIT License - see LICENSE file for details

## Author

Andrey Makarov <viva.la.akam@gmail.com>
