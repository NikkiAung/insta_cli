# grep-file

A simple grep-like command-line tool written in Rust. Searches for a pattern in a file and prints matching lines.

## Usage

```bash
cargo run <query> <filename>
```

### Example

```bash
cargo run to poem.txt
```

This will search for "to" in `poem.txt` and print all matching lines.

## Case Sensitivity

By default, the search is **case-sensitive**. To enable case-insensitive search, set the `CASE_INSENSITIVE` environment variable:

```bash
export CASE_INSENSITIVE=true
cargo run To poem.txt
```

To switch back to case-sensitive mode:

```bash
unset CASE_INSENSITIVE
```

## Running Tests

```bash
cargo test
```

## Project Structure

```
src/
├── main.rs   # Entry point, argument parsing
└── lib.rs    # Core search logic and Config struct
```
