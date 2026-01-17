# claptest

A simple Rust CLI application demonstrating the [clap](https://docs.rs/clap) command-line argument parsing library.

## Features

- Subcommands (`register-person`, `register-pet`)
- Short and long argument flags
- Required arguments
- Boolean flags
- Argument aliases

## Usage

```bash
# Register a person
claptest register-person --first-name John --last-name Doe

# Using short flags
claptest register-person -f John -l Doe

# Using aliases
claptest register-person --fname John --lname Doe

# With the fluffy flag
claptest --fluffy register-person -f John -l Doe

# Register a pet
claptest register-pet --pet-name Buddy
```

## Building

```bash
cargo build --release
```

## Dependencies

- [clap](https://crates.io/crates/clap) v4.5 - Command Line Argument Parser for Rust
