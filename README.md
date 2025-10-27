# Random Number Validator

A simple application to validate and assess the quality of random number sequences using statistical tests.

## Overview

This application consists of:
- **Backend**: Rust-based validation logic that analyzes random number sequences
- **Frontend**: Clean web interface for submitting numbers and viewing results
- **NIST Integration**: Wrapper for the NIST Statistical Test Suite (optional)

## Features

- Parse and validate sequences of random numbers
- Calculate quality metrics including:
  - Bit balance (distribution of 0s and 1s)
  - Runs test (pattern analysis)
  - Overall quality score (0-100%)
- Simple web interface for easy testing
- Extensible architecture for NIST test suite integration

## Quick Start

### Prerequisites

- Rust (1.70 or newer)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd randomnumbervalidator
```

2. Run the application:
```bash
make run
```

Or manually:
```bash
cargo run --bin server
```

3. Open your browser to:
```
http://127.0.0.1:3000
```

4. Enter numbers separated by any delimiter (e.g., `42, 17, 89, 3, 56` or `42\n17\n89\n3\n56`) and click "Validate Numbers"

### Logging

The application includes comprehensive logging. Control log level with the `RUST_LOG` environment variable:

```bash
# Info level (default)
cargo run --bin server

# Debug level (detailed logs)
RUST_LOG=debug cargo run --bin server

# Trace level (very detailed, including HTTP requests)
RUST_LOG=trace cargo run --bin server

# Specific module logging
RUST_LOG=randomnumbervalidator=debug,tower_http=info cargo run --bin server
```

Log output includes:
- Server startup and shutdown events
- HTTP request/response logging
- Validation request details
- NIST test execution progress
- Error conditions and warnings

## Development

### Available Commands

```bash
make help           # Show all available commands
make run            # Build and run the server
make test           # Run all tests
make build          # Build the release version
make clean          # Clean build artifacts
make dev            # Run with auto-reload (requires cargo-watch)
```

### Running Tests

Run all tests:
```bash
make test
```

Or with cargo:
```bash
cargo test
```

Run tests with verbose output:
```bash
cargo test -- --nocapture
```

### Project Structure

```
randomnumbervalidator/
├── src/
│   ├── lib.rs              # Core validation logic
│   ├── nist_wrapper.rs     # NIST test suite wrapper
│   └── bin/
│       └── server.rs       # Web server
├── static/
│   └── index.html          # Frontend interface
├── tests/
│   └── integration_test.rs # Integration tests
├── nist/                   # NIST Statistical Test Suite
│   └── sts-2.1.2/
├── Makefile                # Build and run commands
└── Cargo.toml              # Rust dependencies

```

## How It Works

1. **Input Parsing**: Takes comma-separated integers and converts them to binary representation
2. **Quality Analysis**:
   - Calculates bit balance (equal distribution of 0s and 1s)
   - Performs runs test (checks for patterns)
   - Combines metrics into overall quality score
3. **Results**: Displays quality score and validation status

## NIST Test Suite Integration

The application uses the **NIST Statistical Test Suite** by default for comprehensive randomness analysis.

### Setup NIST Tests

Build the NIST test suite:
```bash
make nist
```

Or manually:
```bash
cd nist/sts-2.1.2/sts-2.1.2
make
```

### Using NIST Tests

**From the Web Interface:**
1. Enter your numbers (minimum 4 numbers for 100+ bits)
2. Click "Validate Numbers"
3. Results will include detailed NIST test analysis

NIST tests are **always enabled** and run automatically on all validations.

**From the API:**
```bash
# NIST tests run by default
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56,91,23,67"}'

# Or explicitly enable (same result)
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56,91,23,67","use_nist":true}'

# Disable NIST (basic tests only)
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56,91,23,67","use_nist":false}'
```

### NIST Test Results

The NIST integration runs 15 different statistical tests including:
- Frequency (Monobit) Test
- Block Frequency Test
- Cumulative Sums Test
- Runs Test
- Longest Run of Ones Test
- Binary Matrix Rank Test
- Discrete Fourier Transform Test
- Non-overlapping Template Matching Test
- Overlapping Template Matching Test
- Maurer's Universal Statistical Test
- Approximate Entropy Test
- Random Excursions Test
- Random Excursions Variant Test
- Serial Test
- Linear Complexity Test

Results include:
- Pass/Fail status for each test
- Average p-values
- Overall success rate

## API Endpoints

### POST `/api/validate`

Validates a sequence of random numbers.

**Request:**
```json
{
  "numbers": "42, 17, 89, 3, 56"
}
```

Note: `use_nist` defaults to `true`. NIST tests run automatically unless explicitly disabled.

**Response:**
```json
{
  "valid": true,
  "quality_score": 0.75,
  "message": "Analyzed 160 bits",
  "nist_results": "NIST Statistical Tests Summary\n================================\nTests Passed: 12/15\nSuccess Rate: 80.0%\n..."
}
```

Response includes:
- Basic quality score (0.0 to 1.0)
- Pass/fail validation
- Detailed NIST test results with pass/fail status and p-values for all 15 tests

## Testing the Application

### From the Web Interface

1. Start the server: `make run`
2. Open http://127.0.0.1:3000
3. Try these examples:
   - Good randomness: `42, 17, 89, 3, 56, 91, 23, 67, 14, 88`
   - Poor randomness: `1, 1, 1, 1, 1, 1, 1, 1`
   - Large sequence: Generate 50+ random numbers

Note: NIST tests run automatically on all validations and may take 3-5 seconds.

### From Command Line (API)

```bash
# NIST tests run by default
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56"}'
```

## Troubleshooting

**Server won't start:**
- Check if port 3000 is available
- Try `cargo clean` and rebuild
- Enable debug logging: `RUST_LOG=debug cargo run --bin server`

**Tests failing:**
- Ensure you're using Rust 1.70 or newer
- Run `cargo update` to get latest dependencies

**NIST tests not working:**
- The NIST test suite requires compilation: `make nist`
- Check logs with `RUST_LOG=debug` to see detailed error messages
- Verify the assess binary exists: `ls nist/sts-2.1.2/sts-2.1.2/assess`

**Debugging issues:**
- Enable detailed logging: `RUST_LOG=debug cargo run --bin server`
- Check logs for error messages and stack traces
- Logs show: request details, validation progress, NIST execution, errors

## Future Enhancements

- [x] ~~Automated NIST test suite integration~~ **COMPLETED**
- [ ] Support for different input formats (binary, hex)
- [ ] Additional custom statistical tests
- [ ] Test result visualization with charts
- [ ] Batch processing capabilities
- [ ] API rate limiting and authentication
- [ ] Caching of NIST results for repeated tests

## License

See LICENSE file for details.