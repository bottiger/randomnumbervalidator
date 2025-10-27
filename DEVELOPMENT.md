# Development Guide

## Quick Start for Development

1. **Run the server:**
   ```bash
   make run
   # or
   cargo run --bin server
   ```

2. **Run tests:**
   ```bash
   make test
   # or
   cargo test
   ```

3. **Check code:**
   ```bash
   cargo check
   cargo clippy
   cargo fmt
   ```

## Architecture

### Backend (Rust)

**Core Files:**
- `src/lib.rs` - Main validation logic
  - `prepare_input_for_nist()` - Converts numbers to binary
  - `validate_random_numbers()` - Main validation function
  - `calculate_basic_quality()` - Quality scoring algorithm

- `src/nist_wrapper.rs` - NIST test suite integration
  - Currently stubbed for future full integration
  - Can check if NIST tools are available
  - Provides file preparation utilities

- `src/bin/server.rs` - Axum web server
  - Serves static HTML at `/`
  - API endpoint at `/api/validate`
  - Runs on port 3000

### Frontend (HTML/JS)

**Files:**
- `static/index.html` - Single-page application
  - Form for number input
  - Result display with quality visualization
  - Clean, gradient UI

**Features:**
- Real-time validation
- Visual quality indicator (progress bar)
- Error handling
- Responsive design

### Tests

**Unit Tests:**
- `src/lib.rs` - Tests for validation logic
- `src/nist_wrapper.rs` - Tests for NIST wrapper

**Integration Tests:**
- `tests/integration_test.rs` - End-to-end validation tests

## API Reference

### POST /api/validate

**Request Body:**
```json
{
  "numbers": "string" // Comma-separated integers
}
```

**Response:**
```json
{
  "valid": boolean,
  "quality_score": number,  // 0.0 to 1.0
  "message": "string",
  "nist_results": "string" | null
}
```

## Quality Scoring Algorithm

The application calculates a quality score based on two metrics:

1. **Bit Balance (0.0 to 1.0)**
   - Measures distribution of 0s and 1s in binary representation
   - Perfect balance = 0.5 of each = score of 1.0
   - All same bit = score of 0.0

2. **Runs Test (0.0 to 1.0)**
   - Counts transitions between consecutive bits
   - Good randomness has many transitions
   - Few transitions indicate patterns

**Final Score:** Average of both metrics

## Adding New Tests

To add a new statistical test:

1. Add function to `src/lib.rs`:
   ```rust
   fn calculate_new_test(bits: &[u8]) -> f64 {
       // Your test logic
       // Return score 0.0 to 1.0
   }
   ```

2. Update `calculate_basic_quality()` to include new test:
   ```rust
   let new_score = calculate_new_test(bits);
   (balance + runs_score + new_score) / 3.0
   ```

3. Add unit tests

## NIST Integration

The NIST Statistical Test Suite is included but requires manual compilation.

**To enable:**
```bash
cd nist/sts-2.1.2/sts-2.1.2
make
```

**Future Work:**
- Automate NIST test execution
- Parse NIST output files
- Display detailed NIST results in UI
- Support batch processing

## Debugging

**Enable logging:**
```rust
// In server.rs, add:
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // ... rest of code
}
```

**Run with output:**
```bash
RUST_LOG=debug cargo run --bin server
```

**Test specific function:**
```bash
cargo test test_name -- --nocapture
```

## Performance Considerations

- Current implementation handles sequences of reasonable size (< 10,000 numbers)
- For larger datasets, consider:
  - Streaming processing
  - Parallel computation
  - Result caching
  - Database storage for results

## Extending the Frontend

The frontend is intentionally minimal. To extend:

1. **Add new input formats:**
   - Binary input
   - Hex input
   - File upload

2. **Enhance visualization:**
   - Bit distribution charts
   - Pattern detection displays
   - Historical comparison

3. **Add features:**
   - Save/load test results
   - Compare multiple sequences
   - Export reports
