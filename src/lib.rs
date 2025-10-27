use serde::{Deserialize, Serialize};

pub mod nist_wrapper;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub numbers: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub quality_score: f64,
    pub message: String,
    pub nist_results: Option<String>,
}

/// Parse the input string and convert to binary format for NIST tests
pub fn prepare_input_for_nist(input: &str) -> Result<Vec<u8>, String> {
    // Parse comma-separated numbers
    let numbers: Result<Vec<u32>, _> = input
        .split(',')
        .map(|s| s.trim().parse::<u32>())
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

/// Validate random numbers and return quality assessment
pub fn validate_random_numbers(input: &str) -> ValidationResponse {
    // Prepare input for NIST
    let bits = match prepare_input_for_nist(input) {
        Ok(b) => b,
        Err(e) => {
            return ValidationResponse {
                valid: false,
                quality_score: 0.0,
                message: e,
                nist_results: None,
            }
        }
    };

    // Basic validation: calculate simple randomness metrics
    let quality_score = calculate_basic_quality(&bits);

    ValidationResponse {
        valid: quality_score > 0.3,
        quality_score,
        message: format!("Analyzed {} bits", bits.len()),
        nist_results: Some("NIST test suite integration pending".to_string()),
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
    let runs_score = 1.0 - ((runs as f64 - expected_runs as f64).abs() / expected_runs as f64).min(1.0);

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
