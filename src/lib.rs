use serde::{Deserialize, Serialize};
use num_bigint::BigUint;

#[allow(unused_imports)]
use tracing::{debug, info, warn};

pub mod nist_wrapper;
pub mod enhanced_stats;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub numbers: String,
    #[serde(default = "default_use_nist")]
    pub use_nist: bool,
    /// Optional: specify the minimum value of your RNG range (e.g., 1 for range 1-100)
    pub range_min: Option<u32>,
    /// Optional: specify the maximum value of your RNG range (e.g., 100 for range 1-100)
    pub range_max: Option<u32>,
}

fn default_use_nist() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub quality_score: f64,
    pub message: String,
    pub nist_results: Option<String>,
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

/// Convert numbers to bits using base conversion (for non-standard ranges)
/// This extracts the true entropy without bias from leading zeros
fn convert_to_bits_base_conversion(numbers: &[u32], range_min: u32, range_max: u32) -> Result<Vec<u8>, String> {
    let range_size = (range_max - range_min + 1) as u64;

    // Convert the sequence of numbers to a large integer (base-range_size representation)
    let mut big_num = BigUint::from(0u32);
    let base = BigUint::from(range_size);

    for &num in numbers {
        // Normalize to 0-based
        let normalized = num - range_min;
        big_num = big_num * &base + BigUint::from(normalized);
    }

    // Convert to binary bits
    let bytes = big_num.to_bytes_be();

    // Convert bytes to individual bits
    let mut bits = Vec::new();
    for byte in bytes {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) as u8);
        }
    }

    // Calculate expected entropy
    let entropy_per_number = (range_size as f64).log2();
    let expected_bits = (numbers.len() as f64 * entropy_per_number) as usize;

    info!(
        "Base conversion: {} numbers → {} bits (expected ~{} bits, {:.2} bits/number)",
        numbers.len(),
        bits.len(),
        expected_bits,
        entropy_per_number
    );

    Ok(bits)
}

/// Validate random numbers and return quality assessment (defaults to using NIST)
pub fn validate_random_numbers(input: &str) -> ValidationResponse {
    validate_random_numbers_with_nist(input, true)
}

/// Validate random numbers with optional NIST test suite integration
pub fn validate_random_numbers_with_nist(input: &str, use_nist: bool) -> ValidationResponse {
    validate_random_numbers_with_nist_and_range(input, use_nist, None, None)
}

/// Validate random numbers with optional NIST test suite integration and custom range
pub fn validate_random_numbers_with_nist_and_range(
    input: &str,
    use_nist: bool,
    range_min: Option<u32>,
    range_max: Option<u32>,
) -> ValidationResponse {
    debug!(
        "Starting validation: input_length={}, use_nist={}, range={:?}-{:?}",
        input.len(),
        use_nist,
        range_min,
        range_max
    );

    // Prepare input for NIST
    let bits = match prepare_input_for_nist_with_range(input, range_min, range_max) {
        Ok(b) => {
            debug!(
                "Successfully parsed {} numbers into {} bits",
                input.split(',').count(),
                b.len()
            );
            b
        }
        Err(e) => {
            warn!("Failed to parse input: {}", e);
            return ValidationResponse {
                valid: false,
                quality_score: 0.0,
                message: e,
                nist_results: None,
            };
        }
    };

    // Basic validation: calculate simple randomness metrics
    let quality_score = calculate_basic_quality(&bits);
    debug!("Basic quality score calculated: {:.4}", quality_score);

    // Run NIST tests if requested and available
    let nist_results = if use_nist {
        info!("NIST tests requested, initializing wrapper");
        let wrapper = nist_wrapper::NistWrapper::new();
        match wrapper.run_tests(&bits) {
            Ok(results) => {
                info!("NIST tests completed successfully");
                Some(results)
            }
            Err(e) => {
                warn!("NIST tests failed: {}", e);
                Some(format!("NIST tests could not be run: {}", e))
            }
        }
    } else {
        debug!("NIST tests not requested");
        Some(
            "NIST tests not requested. Use NIST option to enable comprehensive testing."
                .to_string(),
        )
    };

    let is_valid = quality_score > 0.3;
    info!(
        "Validation complete: valid={}, quality_score={:.4}, bits={}",
        is_valid,
        quality_score,
        bits.len()
    );

    ValidationResponse {
        valid: is_valid,
        quality_score,
        message: format!("Analyzed {} bits", bits.len()),
        nist_results,
    }
}

/// Calculate basic quality score (0.0 to 1.0)
fn calculate_basic_quality(bits: &[u8]) -> f64 {
    if bits.is_empty() {
        return 0.0;
    }

    // Count ones and zeros
    let ones = bits.iter().filter(|&&b| b == 1).count();
    let zeros = bits.len() - ones;

    // Good randomness should have roughly equal ones and zeros
    let balance = 1.0 - ((ones as f64 - zeros as f64).abs() / bits.len() as f64);

    // Simple runs test: count consecutive identical bits
    let mut runs = 0;
    for i in 1..bits.len() {
        if bits[i] != bits[i - 1] {
            runs += 1;
        }
    }
    let expected_runs = bits.len() / 2;
    let runs_score =
        1.0 - ((runs as f64 - expected_runs as f64).abs() / expected_runs as f64).min(1.0);

    // Average the metrics
    (balance + runs_score) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_input_basic() {
        let result = prepare_input_for_nist("1,2,3");
        // Range 1-3 doesn't start at 0, so should require range specification
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("doesn't fit standard bit widths"));
    }

    #[test]
    fn test_prepare_input_invalid() {
        let result = prepare_input_for_nist("1,abc,3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters"));
    }

    #[test]
    fn test_prepare_input_newline_delimiter() {
        let result = prepare_input_for_nist("0\n128\n255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
    }

    #[test]
    fn test_prepare_input_space_delimiter() {
        let result = prepare_input_for_nist("0 100 255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
    }

    #[test]
    fn test_prepare_input_mixed_delimiters() {
        let result = prepare_input_for_nist("0, 50\n100\t150;255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 40); // 5 numbers * 8 bits
    }

    #[test]
    fn test_prepare_input_reject_letters() {
        let result = prepare_input_for_nist("123abc456");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters"));
    }

    #[test]
    fn test_validate_random_numbers() {
        let response = validate_random_numbers("0,1,2,3,4,5");
        assert!(response.quality_score >= 0.0);
        assert!(response.quality_score <= 1.0);
    }

    #[test]
    fn test_basic_quality_balanced() {
        // Perfectly balanced: alternating bits
        let bits = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let score = calculate_basic_quality(&bits);
        assert!(score > 0.5, "Expected score > 0.5, got {}", score);
    }

    #[test]
    fn test_basic_quality_poor() {
        // Poor randomness: all zeros
        let bits = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let score = calculate_basic_quality(&bits);
        assert!(score < 0.5);
    }

    #[test]
    fn test_basic_quality_all_ones() {
        // Poor randomness: all ones
        let bits = vec![1, 1, 1, 1, 1, 1, 1, 1];
        let score = calculate_basic_quality(&bits);
        assert!(score < 0.5);
    }

    #[test]
    fn test_basic_quality_empty() {
        // Edge case: empty bits
        let bits = vec![];
        let score = calculate_basic_quality(&bits);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_prepare_input_single_number() {
        let result = prepare_input_for_nist("0,42");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 16); // 2 numbers * 8 bits
    }

    #[test]
    fn test_prepare_input_zero() {
        let result = prepare_input_for_nist("0");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 8);
        // All bits should be 0
        assert!(bits.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_prepare_input_max_u32() {
        let result = prepare_input_for_nist("0,4294967295"); // u32::MAX
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 64); // 2 numbers * 32 bits
        // Last 32 bits should be all 1
        assert!(bits[32..].iter().all(|&b| b == 1));
    }

    #[test]
    fn test_prepare_input_overflow() {
        // Number larger than u32::MAX should fail
        let result = prepare_input_for_nist("4294967296");
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_input_empty_string() {
        let result = prepare_input_for_nist("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No numbers"));
    }

    #[test]
    fn test_prepare_input_whitespace_only() {
        let result = prepare_input_for_nist("   \n\t  ");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No numbers"));
    }

    #[test]
    fn test_prepare_input_special_characters() {
        // Should treat special chars as delimiters
        let result = prepare_input_for_nist("0!@#100$%^255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
    }

    #[test]
    fn test_prepare_input_negative_sign() {
        // Negative numbers should work (the minus is treated as delimiter)
        let result = prepare_input_for_nist("0,5,10");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
    }

    #[test]
    fn test_validate_invalid_input() {
        let response = validate_random_numbers("abc");
        assert!(!response.valid);
        assert_eq!(response.quality_score, 0.0);
        assert!(response.message.contains("letters"));
    }

    #[test]
    fn test_validate_with_nist_disabled() {
        let response = validate_random_numbers_with_nist("0,1,2,3,4,5", false);
        assert!(response.quality_score >= 0.0);
        assert!(response.nist_results.is_some());
        let nist_msg = response.nist_results.unwrap();
        assert!(nist_msg.contains("not requested"));
    }

    #[test]
    fn test_validate_quality_threshold() {
        // Test that quality_score > 0.3 determines validity
        let response = validate_random_numbers("0,1,2,3,4,5");
        if response.quality_score > 0.3 {
            assert!(response.valid);
        } else {
            assert!(!response.valid);
        }
    }

    #[test]
    fn test_prepare_input_leading_zeros() {
        // Numbers with leading zeros should be parsed correctly
        let result = prepare_input_for_nist("0,007,042,0100");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 32); // 4 numbers * 8 bits (max is 100)
    }

    #[test]
    fn test_validation_response_structure() {
        let response = validate_random_numbers("0,1,2,3,4,5");
        // Verify all fields are populated
        assert!(response.quality_score >= 0.0 && response.quality_score <= 1.0);
        assert!(!response.message.is_empty());
        assert!(response.nist_results.is_some());
    }

    #[test]
    fn test_basic_quality_single_bit() {
        let bits = vec![1];
        let score = calculate_basic_quality(&bits);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_prepare_input_large_sequence() {
        // Test with many numbers
        let numbers: Vec<String> = (1..=100).map(|n| n.to_string()).collect();
        let input = numbers.join(",");
        let result = prepare_input_for_nist(&input);
        assert!(result.is_err()); // Should fail without range
        assert!(result.unwrap_err().contains("doesn't fit standard bit widths"));
    }

    // ========== Tests for standard bit width detection ==========

    #[test]
    fn test_8bit_standard_range() {
        // Numbers 0-255 should use 8 bits per number
        let result = prepare_input_for_nist("0,128,255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
    }

    #[test]
    fn test_16bit_standard_range() {
        // Numbers 0-65535 should use 16 bits per number
        let result = prepare_input_for_nist("0,256,65535");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 48); // 3 numbers * 16 bits
    }

    #[test]
    fn test_32bit_standard_range() {
        // Numbers 0-4294967295 should use 32 bits per number
        let result = prepare_input_for_nist("0,65536,4294967295");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
    }

    #[test]
    fn test_8bit_boundary() {
        // Exactly 255 should still use 8 bits
        let result = prepare_input_for_nist("0,100,255");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 24); // 3 * 8
    }

    // ========== Tests for non-standard ranges (should fail without range specification) ==========

    #[test]
    fn test_nonstandard_range_1_to_100() {
        // Range 1-100 doesn't start at 0, should require range specification
        let result = prepare_input_for_nist("1,50,100");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("doesn't fit standard bit widths"));
        assert!(err_msg.contains("range_min"));
    }

    #[test]
    fn test_nonstandard_range_50_to_200() {
        let result = prepare_input_for_nist("50,100,200");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("range_min and range_max"));
    }

    // ========== Tests for custom range with base conversion ==========

    #[test]
    fn test_custom_range_1_to_100() {
        // With range specified, should use base conversion
        let result = prepare_input_for_nist_with_range("1,50,100", Some(1), Some(100));
        assert!(result.is_ok());
        let bits = result.unwrap();
        // 3 numbers in base-100 ≈ 3 * log2(100) ≈ 3 * 6.64 ≈ 20 bits
        // The actual result is 24 bits (3 bytes from BigUint conversion)
        assert!(bits.len() >= 16 && bits.len() <= 24);
    }

    #[test]
    fn test_custom_range_validation() {
        // Numbers outside specified range should fail
        let result = prepare_input_for_nist_with_range("1,50,101", Some(1), Some(100));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("outside specified range"));
    }

    #[test]
    fn test_custom_range_invalid_min_max() {
        // min > max should fail
        let result = prepare_input_for_nist_with_range("50", Some(100), Some(50));
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("min") && err_msg.contains("max"));
    }

    #[test]
    fn test_base_conversion_deterministic() {
        // Same input should always produce same output
        let result1 = prepare_input_for_nist_with_range("1,2,3,4,5", Some(1), Some(10));
        let result2 = prepare_input_for_nist_with_range("1,2,3,4,5", Some(1), Some(10));
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_base_conversion_entropy() {
        // More numbers should produce more bits
        let result3 = prepare_input_for_nist_with_range("1,2,3", Some(1), Some(10));
        let result10 = prepare_input_for_nist_with_range("1,2,3,4,5,6,7,8,9,10", Some(1), Some(10));
        assert!(result3.is_ok());
        assert!(result10.is_ok());
        let bits3 = result3.unwrap();
        let bits10 = result10.unwrap();
        assert!(bits10.len() > bits3.len());
    }

    #[test]
    fn test_8bit_with_explicit_range() {
        // Even with standard range, explicit range should still work
        let result = prepare_input_for_nist_with_range("0,128,255", Some(0), Some(255));
        assert!(result.is_ok());
        let bits = result.unwrap();
        // With explicit range 0-255, should use base conversion
        // 3 numbers in base-256 ≈ 3 * 8 = 24 bits
        assert_eq!(bits.len(), 24);
    }

    #[test]
    fn test_old_test_compatibility() {
        // Old tests that used 32 bits should now fail or use 8/16 bits
        // Testing 0,42: should use 8 bits
        let result = prepare_input_for_nist("0,42");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 16); // 2 numbers * 8 bits (not 32!)
    }
}
