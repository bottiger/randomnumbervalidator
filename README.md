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

4. Enter comma-separated numbers (e.g., `42, 17, 89, 3, 56`) and click "Validate Numbers"

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

## NIST Test Suite (Optional)

The application includes a wrapper for the NIST Statistical Test Suite for more comprehensive analysis.

To build NIST tests:
```bash
make nist
```

Or manually:
```bash
cd nist/sts-2.1.2/sts-2.1.2
make
```

Note: The NIST test suite requires interactive setup. Full automation is planned for future releases.

## API Endpoints

### POST `/api/validate`

Validates a sequence of random numbers.

**Request:**
```json
{
  "numbers": "42, 17, 89, 3, 56"
}
```

**Response:**
```json
{
  "valid": true,
  "quality_score": 0.75,
  "message": "Analyzed 160 bits",
  "nist_results": "NIST test suite integration pending"
}
```

## Testing the Application

### From the Web Interface

1. Start the server: `make run`
2. Open http://127.0.0.1:3000
3. Try these examples:
   - Good randomness: `42, 17, 89, 3, 56, 91, 23, 67, 14, 88`
   - Poor randomness: `1, 1, 1, 1, 1, 1, 1, 1`
   - Large sequence: Generate 50+ random numbers

### From Command Line (API)

```bash
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56"}'
```

## Troubleshooting

**Server won't start:**
- Check if port 3000 is available
- Try `cargo clean` and rebuild

**Tests failing:**
- Ensure you're using Rust 1.70 or newer
- Run `cargo update` to get latest dependencies

**NIST tests not working:**
- The NIST test suite requires compilation: `make nist`
- Full integration requires additional setup (documented in nist/ directory)

## Future Enhancements

- [ ] Automated NIST test suite integration
- [ ] Support for different input formats (binary, hex)
- [ ] Additional statistical tests
- [ ] Test result visualization
- [ ] Batch processing capabilities
- [ ] API rate limiting and authentication

## License

See LICENSE file for details.