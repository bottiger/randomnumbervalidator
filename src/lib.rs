use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use utoipa::ToSchema;

#[allow(unused_imports)]
use tracing::{debug, info, warn};

pub mod enhanced_stats;
pub mod nist_tests;
pub mod nist_wrapper;

/// Format of the input random numbers.
///
/// Determines how the input string should be parsed before analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    /// Input is a sequence of numbers separated by delimiters (commas, spaces, newlines, etc.).
    ///
    /// # Example
    /// ```text
    /// "1,2,3,4,5"
    /// "42 17 89 12"
    /// ```
    #[default]
    Numbers,

    /// Input is base64-encoded binary data.
    ///
    /// Base64 is decoded to bytes, then converted to individual bits for analysis.
    ///
    /// # Example
    /// ```text
    /// "SGVsbG8gV29ybGQ="
    /// ```
    Base64,
}

/// Request payload for validating random number quality.
///
/// This structure defines all parameters needed to analyze a sequence of random numbers
/// using NIST statistical tests.
///
/// # Example (Numbers)
///
/// ```json
/// {
///   "numbers": "1,2,3,4,5,6,7,8,9,10",
///   "input_format": "numbers",
///   "range_min": 1,
///   "range_max": 100
/// }
/// ```
///
/// # Example (Base64)
///
/// ```json
/// {
///   "numbers": "SGVsbG8gV29ybGQ=",
///   "input_format": "base64"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidationRequest {
    /// The random numbers to validate.
    ///
    /// For `input_format: "numbers"`: comma/space/newline separated numbers.
    /// For `input_format: "base64"`: base64-encoded binary data.
    pub numbers: String,

    /// The format of the input data (default: "numbers").
    ///
    /// Use "numbers" for numeric sequences, or "base64" for binary data.
    #[serde(default)]
    pub input_format: InputFormat,

    /// Minimum value of your RNG range (optional).
    ///
    /// Required for custom ranges that don't fit standard bit widths (0-255, 0-65535, 0-4294967295).
    ///
    /// # Example
    /// For a dice roll (1-6): set `range_min: 1` and `range_max: 6`.
    pub range_min: Option<u32>,

    /// Maximum value of your RNG range (optional).
    ///
    /// Required for custom ranges that don't fit standard bit widths.
    ///
    /// # Example
    /// For a percentage (1-100): set `range_min: 1` and `range_max: 100`.
    pub range_max: Option<u32>,

    /// Enforce a specific bit-width (8, 16, or 32) for fixed-width encoding (optional).
    ///
    /// If specified, all numbers must fit within this bit-width.
    /// Only valid for `input_format: "numbers"`.
    pub bit_width: Option<u8>,

    /// Enable debug logging of the bit stream to a file (default: false).
    ///
    /// When enabled, writes the converted bit stream to a timestamped file in the `debug/` directory.
    #[serde(default)]
    pub debug_log: bool,
}

/// Result of a single NIST statistical test.
///
/// Each test analyzes the randomness of the bit sequence using different statistical methods.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NistTestResult {
    /// Name of the test (e.g., "Frequency", "BlockFrequency", "Runs").
    pub name: String,

    /// Whether the test passed (p-value >= 0.01).
    pub passed: bool,

    /// The p-value of the test (0.0 to 1.0).
    ///
    /// Higher values indicate more random-like behavior. Values >= 0.01 typically pass.
    pub p_value: f64,

    /// Individual p-values for each test iteration (if applicable).
    pub p_values: Vec<f64>,

    /// Human-readable description of what this test measures.
    pub description: String,

    /// Additional test-specific metrics (optional).
    ///
    /// Contains key-value pairs with extra information about the test results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<(String, String)>>,
}

/// Aggregate results from all NIST statistical tests.
///
/// Contains summary statistics and detailed results for each individual test.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NistResults {
    /// Total number of bits analyzed.
    pub bit_count: usize,

    /// Number of tests that passed.
    pub tests_passed: usize,

    /// Total number of tests executed.
    pub total_tests: usize,

    /// Percentage of tests that passed (0.0 to 100.0).
    pub success_rate: f64,

    /// Detailed results for each individual test.
    pub individual_tests: Vec<NistTestResult>,

    /// Optional message if fallback behavior was used.
    pub fallback_message: Option<String>,

    /// Raw output from NIST test suite (optional, for debugging).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_output: Option<String>,
}

/// Response from the random number validation endpoint.
///
/// Contains the validation result, quality score, and detailed test results.
///
/// # Example
///
/// ```json
/// {
///   "valid": true,
///   "quality_score": 0.95,
///   "message": "Analyzed 1000 bits using 15 NIST tests (14/15 passed)",
///   "nist_data": {
///     "bit_count": 1000,
///     "tests_passed": 14,
///     "total_tests": 15,
///     "success_rate": 93.33,
///     "individual_tests": [...]
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidationResponse {
    /// Whether the random numbers are considered valid (quality_score >= 0.8).
    pub valid: bool,

    /// Quality score from 0.0 to 1.0 based on NIST test success rate.
    ///
    /// Calculated as `success_rate / 100`. A score >= 0.8 indicates good quality randomness.
    pub quality_score: f64,

    /// Human-readable summary message.
    pub message: String,

    /// Legacy field: raw NIST output as string (deprecated, for backwards compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nist_results: Option<String>,

    /// Structured NIST test results with detailed metrics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nist_data: Option<NistResults>,

    /// Path to debug bit stream file (only present if debug_log was enabled in request).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_file: Option<String>,
}

/// Parse the input string and convert to binary format for NIST tests
/// Accepts any non-numeric character as delimiter (spaces, commas, newlines, etc.)
/// Rejects input containing letters
///
/// Strategy:
/// - If numbers fit standard ranges (0-255, 0-65535, or 0-4294967295): uses fixed-width bits
/// - If numbers don't fit standard ranges: requires range_min/range_max for base conversion
///
/// This prevents bias from leading zeros when testing numbers in custom ranges.
pub fn prepare_input_for_nist(input: &str) -> Result<Vec<u8>, String> {
    prepare_input_for_nist_with_range(input, None, None)
}

/// Parse and convert to binary with optional custom range specification
pub fn prepare_input_for_nist_with_range(
    input: &str,
    range_min: Option<u32>,
    range_max: Option<u32>,
) -> Result<Vec<u8>, String> {
    // First check for letters (a-z, A-Z) which should be an error
    if input.chars().any(|c| c.is_alphabetic()) {
        return Err("Input contains letters - only numbers and delimiters are allowed".to_string());
    }

    // Extract all sequences of digits, treating everything else as delimiter
    let numbers: Result<Vec<u32>, _> = input
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u32>())
        .collect();

    let nums = match numbers {
        Ok(n) if n.is_empty() => return Err("No numbers provided".to_string()),
        Ok(n) => n,
        Err(_) => return Err("Invalid number format".to_string()),
    };

    let actual_min = *nums.iter().min().unwrap();
    let actual_max = *nums.iter().max().unwrap();

    // Check if numbers fit standard bit widths (with 0 minimum)
    // Note: u32 is always <= 0xFFFF_FFFF, so we only need to check the smaller ranges
    let fits_standard = actual_min == 0;

    if fits_standard {
        // Use fixed-width bit representation
        let bit_width = if actual_max <= 0xFF {
            8
        } else if actual_max <= 0xFFFF {
            16
        } else {
            32
        };

        info!(
            "Using fixed-width: {} bits (range 0-{})",
            bit_width, actual_max
        );

        let mut bits = Vec::new();
        for &num in &nums {
            for i in (0..bit_width).rev() {
                bits.push(((num >> i) & 1) as u8);
            }
        }

        info!(
            "Converted {} numbers to {} bits ({} bits per number)",
            nums.len(),
            bits.len(),
            bit_width
        );

        Ok(bits)
    } else {
        // Numbers don't fit standard ranges - need custom range
        match (range_min, range_max) {
            (Some(min), Some(max)) => {
                // Validate range
                if min > max {
                    return Err(format!("Invalid range: min ({}) > max ({})", min, max));
                }
                if actual_min < min || actual_max > max {
                    return Err(format!(
                        "Numbers ({}-{}) outside specified range ({}-{})",
                        actual_min, actual_max, min, max
                    ));
                }

                info!(
                    "Using base conversion for custom range {}-{} (actual: {}-{})",
                    min, max, actual_min, actual_max
                );

                // Use base conversion to extract unbiased bits
                convert_to_bits_base_conversion(&nums, min, max)
            }
            _ => {
                // No range provided, but numbers don't fit standard ranges
                Err(format!(
                    "Numbers range from {} to {}, which doesn't fit standard bit widths (0-255, 0-65535, or 0-4294967295). \
                     Please specify the intended range of your random number generator using range_min and range_max fields. \
                     For example, if you're generating numbers 1-100, set range_min=1 and range_max=100.",
                    actual_min, actual_max
                ))
            }
        }
    }
}

/// Prepare input for NIST with optional bit-width enforcement
pub fn prepare_input_for_nist_with_range_and_bitwidth(
    input: &str,
    range_min: Option<u32>,
    range_max: Option<u32>,
    bit_width: Option<u8>,
) -> Result<Vec<u8>, String> {
    // Validate bit_width if provided
    if let Some(bw) = bit_width {
        if bw != 8 && bw != 16 && bw != 32 {
            return Err(format!("Invalid bit_width: {}. Must be 8, 16, or 32.", bw));
        }
    }

    // First check for letters (a-z, A-Z) which should be an error
    if input.chars().any(|c| c.is_alphabetic()) {
        return Err("Input contains letters - only numbers and delimiters are allowed".to_string());
    }

    // Extract all sequences of digits, treating everything else as delimiter
    let numbers: Result<Vec<u32>, _> = input
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u32>())
        .collect();

    let nums = match numbers {
        Ok(n) if n.is_empty() => return Err("No numbers provided".to_string()),
        Ok(n) => n,
        Err(_) => return Err("Invalid number format".to_string()),
    };

    let _actual_min = *nums.iter().min().unwrap();
    let actual_max = *nums.iter().max().unwrap();

    // If bit_width is specified, validate and enforce it
    if let Some(bw) = bit_width {
        let max_value = match bw {
            8 => 0xFF,
            16 => 0xFFFF,
            32 => 0xFFFF_FFFF,
            _ => unreachable!(), // Already validated above
        };

        // Check that numbers fit in the specified bit width
        if actual_max > max_value {
            return Err(format!(
                "Number {} exceeds {}-bit maximum value of {}. Please select a larger bit-width or use custom range.",
                actual_max, bw, max_value
            ));
        }

        // Note: We don't reject if min != 0, as a small sample might just not contain 0
        // The statistical tests will detect bias if it exists
        // However, if the range looks systematically constrained (e.g., all numbers 50-200),
        // the user should use custom range, but we let the results speak for themselves

        // Use the specified bit width
        info!(
            "Using enforced bit-width: {} bits (range 0-{})",
            bw, actual_max
        );

        let mut bits = Vec::new();
        for &num in &nums {
            for i in (0..bw).rev() {
                bits.push(((num >> i) & 1) as u8);
            }
        }

        info!(
            "Converted {} numbers to {} bits ({} bits per number)",
            nums.len(),
            bits.len(),
            bw
        );

        return Ok(bits);
    }

    // No bit_width specified, use existing auto-detection logic
    prepare_input_for_nist_with_range(input, range_min, range_max)
}

/// Convert numbers to bits using base conversion (for non-standard ranges)
/// This extracts the true entropy without bias from leading zeros
pub fn convert_to_bits_base_conversion(
    numbers: &[u32],
    range_min: u32,
    range_max: u32,
) -> Result<Vec<u8>, String> {
    let range_size = (range_max - range_min + 1) as u64;

    // Convert the sequence of numbers to a large integer (base-range_size representation)
    let mut big_num = BigUint::from(0u32);
    let base = BigUint::from(range_size);

    for &num in numbers {
        // Normalize to 0-based
        let normalized = num - range_min;
        big_num = big_num * &base + BigUint::from(normalized);
    }

    // Calculate expected entropy and target bit length
    let entropy_per_number = (range_size as f64).log2();
    let expected_bits = (numbers.len() as f64 * entropy_per_number).ceil() as usize;

    // Convert to binary bits
    let bytes = big_num.to_bytes_be();

    // Convert bytes to individual bits
    let mut bits = Vec::new();
    for byte in bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }

    let current_bits = bits.len();

    // Adjust to exactly expected_bits length
    if current_bits < expected_bits {
        // Pad with leading zeros
        let padding_needed = expected_bits - current_bits;
        let mut padded_bits = vec![0; padding_needed];
        padded_bits.extend(bits);
        bits = padded_bits;

        info!(
            "Base conversion: {} numbers → {} bits (padded {} leading zeros, {:.2} bits/number)",
            numbers.len(),
            bits.len(),
            padding_needed,
            entropy_per_number
        );
    } else if current_bits > expected_bits {
        // Trim leading zeros (to_bytes_be() returns whole bytes, may have extra leading zeros)
        let to_trim = current_bits - expected_bits;

        // Verify we're only trimming zeros (sanity check)
        let leading_zeros = bits.iter().take_while(|&&b| b == 0).count();
        if leading_zeros < to_trim {
            return Err(format!(
                "Value too large: need to trim {} bits but only {} leading zeros available",
                to_trim, leading_zeros
            ));
        }

        bits = bits[to_trim..].to_vec();

        info!(
            "Base conversion: {} numbers → {} bits (trimmed {} leading zeros, {:.2} bits/number)",
            numbers.len(),
            bits.len(),
            to_trim,
            entropy_per_number
        );
    } else {
        info!(
            "Base conversion: {} numbers → {} bits ({:.2} bits/number)",
            numbers.len(),
            bits.len(),
            entropy_per_number
        );
    }

    Ok(bits)
}

/// Parse base64 input and convert to bits
/// Base64 decoding produces bytes, which we convert to individual bits
pub fn parse_base64_to_bits(input: &str) -> Result<Vec<u8>, String> {
    use base64::prelude::*;

    // Remove whitespace from base64 input
    let mut clean_input = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    // Add padding if missing (base64 length must be multiple of 4)
    let padding_needed = (4 - (clean_input.len() % 4)) % 4;
    if padding_needed > 0 {
        clean_input.push_str(&"=".repeat(padding_needed));
        info!(
            "Added {} padding character(s) to base64 input",
            padding_needed
        );
    }

    // Decode base64
    let bytes = BASE64_STANDARD
        .decode(clean_input.as_bytes())
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    if bytes.is_empty() {
        return Err("Base64 decoded to empty data".to_string());
    }

    // Convert bytes to individual bits
    let mut bits = Vec::new();
    for &byte in &bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }

    info!(
        "Decoded {} bytes from base64 → {} bits",
        bytes.len(),
        bits.len()
    );

    Ok(bits)
}

/// Write bits to a debug file for inspection
/// Returns the path to the written file
pub fn write_bits_to_debug_file(bits: &[u8]) -> Result<String, String> {
    // Create debug directory if it doesn't exist
    let debug_dir = Path::new("debug");
    std::fs::create_dir_all(debug_dir)
        .map_err(|e| format!("Failed to create debug directory: {}", e))?;

    // Generate unique timestamped filename (with microseconds to avoid race conditions in tests)
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d_%H%M%S");
    let micros = now.timestamp_subsec_micros();
    let filename = format!("bits_{}_{:06}.txt", timestamp, micros);
    let filepath = debug_dir.join(&filename);

    // Write bits to file
    let mut file =
        File::create(&filepath).map_err(|e| format!("Failed to create debug file: {}", e))?;

    // Write header
    writeln!(file, "# Bit Stream Debug Output")
        .map_err(|e| format!("Failed to write to debug file: {}", e))?;
    writeln!(file, "# Total bits: {}", bits.len())
        .map_err(|e| format!("Failed to write to debug file: {}", e))?;
    writeln!(file, "# Timestamp: {}", chrono::Utc::now())
        .map_err(|e| format!("Failed to write to debug file: {}", e))?;
    writeln!(file, "#").map_err(|e| format!("Failed to write to debug file: {}", e))?;

    // Write bits in groups of 64 for readability
    for (i, chunk) in bits.chunks(64).enumerate() {
        let bit_string: String = chunk
            .iter()
            .map(|&b| if b == 1 { '1' } else { '0' })
            .collect();
        writeln!(file, "{:08}: {}", i * 64, bit_string)
            .map_err(|e| format!("Failed to write to debug file: {}", e))?;
    }

    let path_str = filepath.to_string_lossy().to_string();
    info!("Wrote {} bits to debug file: {}", bits.len(), path_str);

    Ok(path_str)
}

/// Prepare input based on format (numbers or base64) and optional parameters
pub fn prepare_input_with_format(
    input: &str,
    format: &InputFormat,
    range_min: Option<u32>,
    range_max: Option<u32>,
    bit_width: Option<u8>,
) -> Result<Vec<u8>, String> {
    match format {
        InputFormat::Numbers => {
            // Use existing number parsing logic
            prepare_input_for_nist_with_range_and_bitwidth(input, range_min, range_max, bit_width)
        }
        InputFormat::Base64 => {
            // Base64 parsing doesn't use range or bit_width parameters
            if range_min.is_some() || range_max.is_some() || bit_width.is_some() {
                warn!("range_min, range_max, and bit_width are ignored for base64 input");
            }
            parse_base64_to_bits(input)
        }
    }
}

/// Validate random numbers and return quality assessment (always uses NIST)
pub fn validate_random_numbers(input: &str) -> ValidationResponse {
    validate_random_numbers_full(input, &InputFormat::Numbers, None, None, None, false)
}

/// Validate random numbers with full control over all parameters (always uses NIST)
pub fn validate_random_numbers_full(
    input: &str,
    input_format: &InputFormat,
    range_min: Option<u32>,
    range_max: Option<u32>,
    bit_width: Option<u8>,
    debug_log: bool,
) -> ValidationResponse {
    debug!(
        "Starting validation: input_length={}, format={:?}, range={:?}-{:?}, bit_width={:?}, debug_log={}",
        input.len(),
        input_format,
        range_min,
        range_max,
        bit_width,
        debug_log
    );

    // Prepare input based on format
    let bits = match prepare_input_with_format(input, input_format, range_min, range_max, bit_width)
    {
        Ok(b) => {
            debug!("Successfully parsed input into {} bits", b.len());
            b
        }
        Err(e) => {
            warn!("Failed to parse input: {}", e);
            return ValidationResponse {
                valid: false,
                quality_score: 0.0,
                message: e,
                nist_results: None,
                nist_data: None,
                debug_file: None,
            };
        }
    };

    // Write debug log if requested
    let debug_file = if debug_log {
        match write_bits_to_debug_file(&bits) {
            Ok(path) => Some(path),
            Err(e) => {
                warn!("Failed to write debug file: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Run NIST tests (always required)
    info!("Running NIST statistical tests");
    let wrapper = nist_wrapper::NistWrapper::new();
    let nist_data = match wrapper.run_tests(&bits) {
        Ok(results) => {
            info!("NIST tests completed successfully");
            results
        }
        Err(e) => {
            warn!("NIST tests failed: {}", e);
            return ValidationResponse {
                valid: false,
                quality_score: 0.0,
                message: format!("NIST tests failed: {}", e),
                nist_results: None,
                nist_data: None,
                debug_file,
            };
        }
    };

    // Calculate quality score from NIST results (success_rate / 100)
    let quality_score = nist_data.success_rate / 100.0;
    let is_valid = quality_score >= 0.8; // Require 80% of tests to pass

    info!(
        "Validation complete: valid={}, quality_score={:.4}, bits={}, tests_passed={}/{}",
        is_valid,
        quality_score,
        bits.len(),
        nist_data.tests_passed,
        nist_data.total_tests
    );

    ValidationResponse {
        valid: is_valid,
        quality_score,
        message: format!(
            "Analyzed {} bits using {} NIST tests ({}/{} passed)",
            bits.len(),
            nist_data.total_tests,
            nist_data.tests_passed,
            nist_data.total_tests
        ),
        nist_results: nist_data.raw_output.clone(),
        nist_data: Some(nist_data),
        debug_file,
    }
}
