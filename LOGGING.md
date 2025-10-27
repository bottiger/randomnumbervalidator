# Logging Guide

## Overview

The Random Number Validator includes comprehensive structured logging using the `tracing` framework. Logs provide visibility into application behavior, request processing, and error conditions.

## Log Levels

The application supports standard log levels:

- **ERROR**: Critical errors that need immediate attention
- **WARN**: Warning conditions that may require investigation
- **INFO**: General informational messages (default level)
- **DEBUG**: Detailed diagnostic information
- **TRACE**: Very detailed trace information (includes HTTP request/response)

## Configuration

### Basic Usage

```bash
# Default (INFO level)
cargo run --bin server

# Debug level
RUST_LOG=debug cargo run --bin server

# Trace level (very verbose)
RUST_LOG=trace cargo run --bin server
```

### Module-Specific Logging

Control logging for specific modules:

```bash
# Debug for app, info for HTTP
RUST_LOG=randomnumbervalidator=debug,tower_http=info cargo run --bin server

# Debug for NIST wrapper only
RUST_LOG=randomnumbervalidator::nist_wrapper=debug cargo run --bin server

# Trace everything
RUST_LOG=trace cargo run --bin server
```

## Log Output Examples

### Server Startup (INFO level)

```
2025-10-27T10:00:00.000Z  INFO randomnumbervalidator: Starting Random Number Validator server
2025-10-27T10:00:00.001Z  INFO randomnumbervalidator: Server listening on http://127.0.0.1:3000
```

### Validation Request (INFO level)

```
2025-10-27T10:01:00.000Z  INFO randomnumbervalidator: Serving index page
2025-10-27T10:01:05.000Z  INFO randomnumbervalidator: Validation request received: 8 numbers, NIST=false
2025-10-27T10:01:05.010Z  INFO randomnumbervalidator: Validation complete: valid=true, quality_score=0.7234, bits=256
2025-10-27T10:01:05.011Z  INFO randomnumbervalidator: Validation successful: quality_score=0.72
```

### NIST Test Execution (INFO level)

```
2025-10-27T10:02:00.000Z  INFO randomnumbervalidator: Validation request received: 10 numbers, NIST=true
2025-10-27T10:02:00.001Z  INFO randomnumbervalidator: NIST tests requested, initializing wrapper
2025-10-27T10:02:00.002Z  INFO randomnumbervalidator::nist_wrapper: Starting NIST statistical tests
2025-10-27T10:02:00.003Z  INFO randomnumbervalidator::nist_wrapper: Running NIST tests on 320 bits
2025-10-27T10:02:05.234Z  INFO randomnumbervalidator::nist_wrapper: NIST tests completed successfully, parsing results
2025-10-27T10:02:05.250Z  INFO randomnumbervalidator: NIST tests completed successfully
2025-10-27T10:02:05.251Z  INFO randomnumbervalidator: Validation complete: valid=true, quality_score=0.6543, bits=320
```

### Debug Level Output

With `RUST_LOG=debug`, you get additional details:

```
2025-10-27T10:03:00.000Z DEBUG randomnumbervalidator: Starting validation: input_length=47, use_nist=true
2025-10-27T10:03:00.001Z DEBUG randomnumbervalidator: Successfully parsed 10 numbers into 320 bits
2025-10-27T10:03:00.002Z DEBUG randomnumbervalidator: Basic quality score calculated: 0.6543
2025-10-27T10:03:00.003Z  INFO randomnumbervalidator: NIST tests requested, initializing wrapper
2025-10-27T10:03:00.004Z  INFO randomnumbervalidator::nist_wrapper: Starting NIST statistical tests
2025-10-27T10:03:00.005Z DEBUG randomnumbervalidator::nist_wrapper: Project root: /Users/user/dev/randomnumbervalidator
2025-10-27T10:03:00.006Z DEBUG randomnumbervalidator::nist_wrapper: NIST path: /Users/user/dev/randomnumbervalidator/nist/sts-2.1.2/sts-2.1.2
2025-10-27T10:03:00.007Z  INFO randomnumbervalidator::nist_wrapper: Running NIST tests on 320 bits
2025-10-27T10:03:00.008Z DEBUG randomnumbervalidator::nist_wrapper: Preparing input file: test_data_1730026980.txt
2025-10-27T10:03:00.010Z DEBUG randomnumbervalidator::nist_wrapper: Input file written: 320 bits
2025-10-27T10:03:00.011Z DEBUG randomnumbervalidator::nist_wrapper: Spawning NIST assess process
2025-10-27T10:03:00.012Z DEBUG randomnumbervalidator::nist_wrapper: Waiting for NIST assess to complete
2025-10-27T10:03:05.230Z DEBUG randomnumbervalidator::nist_wrapper: NIST assess completed with status: exit status: 1
2025-10-27T10:03:05.231Z  INFO randomnumbervalidator::nist_wrapper: NIST tests completed successfully, parsing results
```

### Error Handling

Errors are logged with full context:

```
2025-10-27T10:04:00.000Z  WARN randomnumbervalidator: Failed to parse input: Invalid number format
2025-10-27T10:04:00.001Z  WARN randomnumbervalidator: Validation failed: quality_score=0.00, reason=Invalid number format

# NIST errors
2025-10-27T10:05:00.000Z  INFO randomnumbervalidator::nist_wrapper: Starting NIST statistical tests
2025-10-27T10:05:00.001Z  WARN randomnumbervalidator::nist_wrapper: Insufficient bits for NIST tests: 96 < 100
2025-10-27T10:05:00.002Z  WARN randomnumbervalidator: NIST tests failed: Need at least 100 bits for NIST tests
```

## HTTP Request Logging

With `tower_http` tracing enabled (INFO or DEBUG level), you get HTTP request/response logging:

```
2025-10-27T10:06:00.000Z  INFO tower_http::trace::on_request: started processing request
2025-10-27T10:06:00.001Z  INFO tower_http::trace::on_response: finished processing request latency=1ms status=200
```

With TRACE level:
```
2025-10-27T10:07:00.000Z TRACE tower_http::trace::on_request: started processing request method=POST uri=/api/validate
2025-10-27T10:07:00.100Z TRACE tower_http::trace::on_response: finished processing request method=POST uri=/api/validate status=200 latency=100ms
```

## Production Recommendations

### Basic Production Setup

```bash
# Production: INFO level only
RUST_LOG=info cargo run --release --bin server
```

### Debugging Production Issues

```bash
# Temporarily enable debug for specific module
RUST_LOG=randomnumbervalidator=debug,tower_http=info cargo run --release --bin server
```

### Log Output to File

```bash
# Redirect logs to file
RUST_LOG=info cargo run --bin server 2>&1 | tee server.log

# Or with systemd
RUST_LOG=info cargo run --bin server 2>&1 | systemd-cat -t randomnumbervalidator
```

## Structured Logging

All logs are structured and can be parsed:

```rust
// In code, logs are structured:
info!(
    "Validation complete: valid={}, quality_score={:.4}, bits={}",
    is_valid, quality_score, bits.len()
);
```

Output format:
```
2025-10-27T10:08:00.000Z  INFO randomnumbervalidator: Validation complete: valid=true, quality_score=0.7234, bits=256
```

## Log Filtering Examples

### Show Only Errors

```bash
RUST_LOG=error cargo run --bin server
```

### Debug Application, Warn for Dependencies

```bash
RUST_LOG=randomnumbervalidator=debug,warn cargo run --bin server
```

### Trace Specific Function

```bash
RUST_LOG=randomnumbervalidator::nist_wrapper::run_tests=trace cargo run --bin server
```

## Performance Considerations

- **INFO level**: Minimal performance impact, recommended for production
- **DEBUG level**: Slight performance impact, useful for troubleshooting
- **TRACE level**: Higher performance impact, use only for detailed debugging

## Integration with Monitoring Tools

### JSON Output

For integration with log aggregation tools, consider using JSON formatting:

```rust
// Add to Cargo.toml:
// tracing-subscriber = { version = "0.3", features = ["json"] }

// In server.rs, replace fmt::layer() with:
// .with(tracing_subscriber::fmt::layer().json())
```

### OpenTelemetry

The `tracing` framework supports OpenTelemetry for distributed tracing (not currently implemented).

## Troubleshooting Logging

**No logs appearing:**
- Check `RUST_LOG` is set correctly
- Ensure logs aren't being filtered by terminal/shell
- Try explicit `RUST_LOG=trace` to see everything

**Too many logs:**
- Reduce log level: `RUST_LOG=warn` or `RUST_LOG=error`
- Filter specific modules: `RUST_LOG=randomnumbervalidator=info,tower_http=warn`

**Logs not helpful:**
- Increase detail: `RUST_LOG=debug`
- Check specific module: `RUST_LOG=randomnumbervalidator::nist_wrapper=debug`
