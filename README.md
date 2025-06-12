# stellar-contracts--wasms

A Rust application that interfaces with stellar-core to process XDR metadata output and convert it to JSON format.

## Features

- Starts stellar-core as a subprocess with metadata mode enabled
- Configures stellar-core for testnet operation
- Processes `Frame<LedgerCloseMeta>` XDR values from stellar-core's stdout
- Decodes XDR using the `stellar-xdr` crate
- Outputs processed data as JSON

## Usage

### Prerequisites

- Rust toolchain (1.70+)
- stellar-core binary available in your PATH or specify with `--stellar-core-path`

### Building

```bash
cargo build --release
```

### Running

```bash
# Use default stellar-core binary and config
./target/release/stellar-core-xdr-processor

# Specify custom stellar-core path
./target/release/stellar-core-xdr-processor --stellar-core-path /path/to/stellar-core

# Use custom config file
./target/release/stellar-core-xdr-processor --config-path /path/to/stellar-core.cfg

# Enable verbose output
./target/release/stellar-core-xdr-processor --verbose
```

### Command Line Options

- `--stellar-core-path <PATH>`: Path to stellar-core binary (default: "stellar-core")
- `--config-path <PATH>`: Path to stellar-core configuration file (default: "stellar-core-testnet.cfg")
- `--verbose`: Enable verbose output for debugging
- `--help`: Show help information

## Configuration

The application includes a default testnet configuration file (`stellar-core-testnet.cfg`) that:

- Configures stellar-core for the Stellar testnet
- Enables metadata output to stdout
- Sets up testnet validators and quorum configuration
- Configures history archives

## Output Format

The application outputs JSON objects for each processed XDR frame:

```json
{
  "frame_type": "LedgerCloseMeta",
  "processed_at": 1699123456,
  "processing_status": "success",
  "note": "XDR frame successfully decoded from stellar-core metadata output"
}
```

## Dependencies

- `stellar-xdr`: For XDR parsing and handling
- `tokio`: For async runtime and process management
- `serde_json`: For JSON serialization
- `clap`: For command-line argument parsing
- `anyhow`: For error handling
- `base64`: For base64 decoding
- `chrono`: For timestamp handling