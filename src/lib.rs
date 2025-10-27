use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use tracing::{debug, info, warn};

pub mod nist_wrapper;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub numbers: String,
    #[serde(default = "default_use_nist")]
    pub use_nist: bool,
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
pub fn prepare_input_for_nist(input: &str) -> Result<Vec<u8>, String> {
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

    match numbers {
        Ok(nums) => {
            if nums.is_empty() {
                return Err("No numbers provided".to_string());
            }
            // Convert to binary representation (simple implementation)
            let mut bits = Vec::new();
            for num in nums {
                // Convert each number to binary bits
                for i in (0..32).rev() {
                    bits.push(((num >> i) & 1) as u8);
                }
            }
            Ok(bits)
        }
        Err(_) => Err("Invalid number format".to_string()),
    }
}

/// Validate random numbers and return quality assessment (defaults to using NIST)
pub fn validate_random_numbers(input: &str) -> ValidationResponse {
    validate_random_numbers_with_nist(input, true)
}

/// Validate random numbers with optional NIST test suite integration
pub fn validate_random_numbers_with_nist(input: &str, use_nist: bool) -> ValidationResponse {
    debug!(
        "Starting validation: input_length={}, use_nist={}",
        input.len(),
        use_nist
    );

    // Prepare input for NIST
    let bits = match prepare_input_for_nist(input) {
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
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
    }

    #[test]
    fn test_prepare_input_invalid() {
        let result = prepare_input_for_nist("1,abc,3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters"));
    }

    #[test]
    fn test_prepare_input_newline_delimiter() {
        let result = prepare_input_for_nist("1\n2\n3");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
    }

    #[test]
    fn test_prepare_input_space_delimiter() {
        let result = prepare_input_for_nist("1 2 3");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
    }

    #[test]
    fn test_prepare_input_mixed_delimiters() {
        let result = prepare_input_for_nist("1, 2\n3\t4;5");
        assert!(result.is_ok());
        let bits = result.unwrap();
        assert_eq!(bits.len(), 160); // 5 numbers * 32 bits
    }

    #[test]
    fn test_prepare_input_reject_letters() {
        let result = prepare_input_for_nist("123abc456");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters"));
    }

    #[test]
    fn test_validate_random_numbers() {
        let response = validate_random_numbers("1,2,3,4,5");
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
}
