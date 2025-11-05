use nistrs::prelude::*;
use std::collections::HashMap;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use crate::{NistResults, NistTestResult};

/// Test tier information based on input size
#[derive(Debug, Clone)]
struct TestTier {
    level: u8,
    name: &'static str,
    description: &'static str,
    min_bits: usize,
    recommended_bits: usize,
}

impl TestTier {
    /// Tier 1: Minimal (100+ bits) - Basic frequency and runs tests only
    const TIER_1: TestTier = TestTier {
        level: 1,
        name: "Minimal",
        description: "Basic tests (Frequency, Runs, FFT)",
        min_bits: 100,
        recommended_bits: 1_000,
    };

    /// Tier 2: Light (1,000+ bits) - Add block-based and template tests
    const TIER_2: TestTier = TestTier {
        level: 2,
        name: "Light",
        description: "Basic + Block tests",
        min_bits: 1_000,
        recommended_bits: 10_000,
    };

    /// Tier 3: Standard (10,000+ bits) - Most tests available
    const TIER_3: TestTier = TestTier {
        level: 3,
        name: "Standard",
        description: "Most NIST tests",
        min_bits: 10_000,
        recommended_bits: 100_000,
    };

    /// Tier 4: Full (100,000+ bits) - All tests with reliable statistics
    const TIER_4: TestTier = TestTier {
        level: 4,
        name: "Full",
        description: "Complete NIST suite",
        min_bits: 100_000,
        recommended_bits: 1_000_000,
    };

    /// Tier 5: Comprehensive (1,000,000+ bits) - Maximum statistical reliability
    const TIER_5: TestTier = TestTier {
        level: 5,
        name: "Comprehensive",
        description: "Full suite with optimal reliability",
        min_bits: 1_000_000,
        recommended_bits: 10_000_000,
    };
}

/// Wrapper for NIST Statistical Test Suite using nistrs crate
pub struct NistWrapper {
    // No need for paths anymore - tests run in-memory
}

impl NistWrapper {
    pub fn new() -> Self {
        NistWrapper {}
    }

    /// Check if NIST test suite is available (always true with nistrs)
    pub fn is_available(&self) -> bool {
        true
    }

    /// Determine which test tier to use based on input size
    fn determine_tier(bit_count: usize) -> TestTier {
        if bit_count >= TestTier::TIER_5.min_bits {
            TestTier::TIER_5
        } else if bit_count >= TestTier::TIER_4.min_bits {
            TestTier::TIER_4
        } else if bit_count >= TestTier::TIER_3.min_bits {
            TestTier::TIER_3
        } else if bit_count >= TestTier::TIER_2.min_bits {
            TestTier::TIER_2
        } else if bit_count >= TestTier::TIER_1.min_bits {
            TestTier::TIER_1
        } else {
            // Return error tier with details
            TestTier {
                level: 0,
                name: "Insufficient",
                description: "Too few bits for NIST tests",
                min_bits: 0,
                recommended_bits: TestTier::TIER_1.min_bits,
            }
        }
    }

    /// Run NIST test suite directly on the bits
    /// Returns structured test results
    pub fn run_tests(&self, bits: &[u8]) -> Result<NistResults, String> {
        self.run_tests_structured(bits)
    }

    /// Run NIST tests and return structured data
    pub fn run_tests_structured(&self, bits: &[u8]) -> Result<NistResults, String> {
        info!("Starting NIST statistical tests with nistrs");

        // Determine test tier based on input size
        let tier = Self::determine_tier(bits.len());

        // Check if we have enough data
        if tier.level == 0 {
            warn!("Insufficient bits for NIST tests: {} < {}", bits.len(), TestTier::TIER_1.min_bits);
            return Err(format!(
                "NIST statistical tests require at least {} bits (~{} numbers with 32-bit encoding) for basic tests. \
                 You provided {} bits (~{} numbers). The system will use enhanced statistical tests instead, \
                 which are designed for smaller datasets.",
                TestTier::TIER_1.min_bits,
                TestTier::TIER_1.min_bits / 32,
                bits.len(),
                bits.len() / 32
            ));
        }

        info!("Running NIST tests on {} bits (Tier {}: {})", bits.len(), tier.level, tier.name);

        // Convert Vec<u8> (0s and 1s) to packed bytes for nistrs
        let packed_bytes = Self::pack_bits_to_bytes(bits);
        let bits_data = BitsData::from_binary(packed_bytes);

        // Run tests appropriate for this tier
        let test_results = Self::run_all_tests(&bits_data, &tier);

        // Parse results into structured format
        self.parse_test_results(bits, test_results, &tier)
    }

    /// Convert Vec<u8> where each element is 0 or 1 into packed bytes
    fn pack_bits_to_bytes(bits: &[u8]) -> Vec<u8> {
        let mut packed = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_count = 0;

        for &bit in bits {
            current_byte = (current_byte << 1) | (bit & 1);
            bit_count += 1;

            if bit_count == 8 {
                packed.push(current_byte);
                current_byte = 0;
                bit_count = 0;
            }
        }

        // Handle remaining bits
        if bit_count > 0 {
            // Pad with zeros on the right
            current_byte <<= 8 - bit_count;
            packed.push(current_byte);
        }

        packed
    }

    /// Run NIST tests appropriate for the given tier
    fn run_all_tests(data: &BitsData, tier: &TestTier) -> HashMap<String, Vec<TestResultT>> {
        let mut results = HashMap::new();

        // Tier 1: Basic tests (100+ bits)
        if tier.level >= 1 {
            results.insert("Frequency".to_string(), vec![frequency_test(data)]);
            results.insert("Runs".to_string(), vec![runs_test(data)]);
            results.insert("FFT".to_string(), vec![fft_test(data)]);

            // Universal test requires at least 1,000 bits to avoid overflow in nistrs
            if data.len() >= 1000 {
                results.insert("Universal".to_string(), vec![universal_test(data)]);
            }

            // Cumulative Sums (returns [TestResultT; 2])
            let cusum_results = cumulative_sums_test(data);
            results.insert("CumulativeSums-Forward".to_string(), vec![cusum_results[0]]);
            results.insert("CumulativeSums-Reverse".to_string(), vec![cusum_results[1]]);
        }

        // Tier 2: Add block and template tests (1,000+ bits)
        if tier.level >= 2 {
            let block_size = if data.len() >= 1000 { 100 } else { data.len() / 10 };
            if block_size > 0 {
                if let Ok(result) = block_frequency_test(data, block_size) {
                    results.insert("BlockFrequency".to_string(), vec![result]);
                }
            }

            // Template tests
            let template_size = 9; // Standard NIST template size
            if let Ok(result_vec) = non_overlapping_template_test(data, template_size) {
                results.insert("NonOverlappingTemplate".to_string(), result_vec);
            }
            let result = overlapping_template_test(data, template_size);
            results.insert("OverlappingTemplate".to_string(), vec![result]);
        }

        // Tier 3: Add more complex tests (10,000+ bits)
        if tier.level >= 3 {
            if let Ok(result) = longest_run_of_ones_test(data) {
                results.insert("LongestRun".to_string(), vec![result]);
            }
            if let Ok(result) = rank_test(data) {
                results.insert("Rank".to_string(), vec![result]);
            }

            // Approximate Entropy (use m=10 as recommended)
            let m_param = 10.min(data.len() / 100);
            if m_param >= 2 {
                let result = approximate_entropy_test(data, m_param);
                results.insert("ApproximateEntropy".to_string(), vec![result]);
            }

            // Serial test (returns [TestResultT; 2])
            let serial_m = 16.min(data.len() / 100);
            if serial_m >= 2 {
                let serial_results = serial_test(data, serial_m);
                results.insert("Serial-1".to_string(), vec![serial_results[0]]);
                results.insert("Serial-2".to_string(), vec![serial_results[1]]);
            }
        }

        // Tier 4: Add heavy tests requiring substantial data (100,000+ bits)
        if tier.level >= 4 {
            // Random Excursions test (returns Result<[(bool, f64); 8], String>)
            if let Ok(excursions_results) = random_excursions_test(data) {
                results.insert("RandomExcursions".to_string(), excursions_results.to_vec());
            }

            // Random Excursions Variant (returns Result<[(bool, f64); 18], String>)
            if let Ok(variant_results) = random_excursions_variant_test(data) {
                results.insert("RandomExcursionsVariant".to_string(), variant_results.to_vec());
            }

            // Linear Complexity (use block size) - returns (bool, f64) directly
            let lc_block_size = 500.min(data.len() / 100);
            if lc_block_size >= 100 {
                let result = linear_complexity_test(data, lc_block_size);
                results.insert("LinearComplexity".to_string(), vec![result]);
            }
        }

        results
    }

    /// Parse test results into structured format
    fn parse_test_results(
        &self,
        bits: &[u8],
        test_results: HashMap<String, Vec<TestResultT>>,
        tier: &TestTier,
    ) -> Result<NistResults, String> {
        let bit_count = bits.len();
        let mut success_count = 0;
        let mut total_tests = 0;

        // Build individual test results
        let mut individual_tests = Vec::new();
        for (test_name, results_vec) in &test_results {
            // Each test may have multiple sub-results
            for (i, (passed, p_value)) in results_vec.iter().enumerate() {
                total_tests += 1;
                if *passed {
                    success_count += 1;
                }

                // If multiple results, add index to name
                let name = if results_vec.len() > 1 {
                    format!("{}-{}", test_name, i + 1)
                } else {
                    test_name.clone()
                };

                individual_tests.push(NistTestResult {
                    name,
                    passed: *passed,
                    p_value: *p_value,
                    p_values: vec![*p_value],
                    description: format!("P-value: {:.4}", p_value),
                    metrics: None,
                });
            }
        }

        // Sort tests by name for consistent display
        individual_tests.sort_by(|a, b| a.name.cmp(&b.name));

        // Calculate success rate
        let success_rate = if total_tests > 0 {
            (success_count as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        // Generate raw output text with tier information
        let raw_output = Self::generate_raw_output(bit_count, &individual_tests, success_count, total_tests, success_rate, tier);

        info!("NIST tests completed (Tier {}): {}/{} passed ({:.1}%)", tier.level, success_count, total_tests, success_rate);

        Ok(NistResults {
            bit_count,
            tests_passed: success_count,
            total_tests,
            success_rate,
            individual_tests,
            fallback_message: None,
            raw_output: Some(raw_output),
        })
    }

    /// Generate raw output text for display
    fn generate_raw_output(
        bit_count: usize,
        individual_tests: &[NistTestResult],
        success_count: usize,
        total_tests: usize,
        success_rate: f64,
        tier: &TestTier,
    ) -> String {
        let mut output = format!(
            "NIST Statistical Test Suite - Results\n\
             ======================================\n\n\
             Dataset: {} bits\n\
             Test Tier: Level {} - {} ({})\n\n\
             Overall: {}/{} tests passed ({:.1}%)\n\n\
             Individual Test Results:\n\
             ------------------------\n",
            bit_count, tier.level, tier.name, tier.description, success_count, total_tests, success_rate
        );

        for test in individual_tests {
            output.push_str(&format!(
                "  {} {}: p-value = {:.6}\n",
                if test.passed { "✓" } else { "✗" },
                test.name,
                test.p_value
            ));
        }

        output.push_str("\n\nAll tests use significance level α = 0.01\n");
        output.push_str("Tests pass if p-value ≥ 0.01\n\n");

        // Add tier guidance
        output.push_str("Test Coverage:\n");
        output.push_str("-------------\n");
        if tier.level < 5 {
            let next_tier = match tier.level {
                1 => TestTier::TIER_2,
                2 => TestTier::TIER_3,
                3 => TestTier::TIER_4,
                4 => TestTier::TIER_5,
                _ => TestTier::TIER_5,
            };
            output.push_str(&format!(
                "Current: Tier {} ({}) - {} tests run\n\
                 Next Tier: Level {} ({}) requires {} bits (~{} numbers)\n",
                tier.level,
                tier.description,
                total_tests,
                next_tier.level,
                next_tier.name,
                next_tier.min_bits,
                next_tier.min_bits / 32
            ));
        } else {
            output.push_str(&format!(
                "Maximum tier reached (Tier {}). All NIST tests available with optimal reliability.\n",
                tier.level
            ));
        }

        output
    }

    /// Parse NIST results from a specific directory (for backwards compatibility)
    pub fn parse_results(&self, _results_dir: &str) -> Result<NistResults, String> {
        // Legacy method - not used with nistrs
        Err("parse_results() is deprecated with nistrs integration".to_string())
    }
}

impl Default for NistWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_wrapper_creation() {
        let wrapper = NistWrapper::new();
        // Verify we can create the wrapper
        assert!(wrapper.is_available());
    }

    #[test]
    fn test_is_available() {
        let wrapper = NistWrapper::new();
        // nistrs is always available
        assert!(wrapper.is_available());
    }

    #[test]
    fn test_run_tests_insufficient_bits() {
        let wrapper = NistWrapper::new();
        let bits = vec![1, 0, 1, 0]; // Only 4 bits, need at least 100,000

        let result = wrapper.run_tests(&bits);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("100000") || error_msg.contains("require"));
    }

    #[test]
    fn test_nist_wrapper_default() {
        let wrapper = NistWrapper::default();
        assert!(wrapper.is_available());
    }

    #[test]
    fn test_pack_bits_to_bytes() {
        // Test: 8 bits should pack into 1 byte
        let bits = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let packed = NistWrapper::pack_bits_to_bytes(&bits);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0b10101010);
    }

    #[test]
    fn test_pack_bits_to_bytes_partial() {
        // Test: 12 bits should pack into 2 bytes (last byte padded)
        let bits = vec![1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0];
        let packed = NistWrapper::pack_bits_to_bytes(&bits);
        assert_eq!(packed.len(), 2);
        assert_eq!(packed[0], 0b11110000);
        assert_eq!(packed[1], 0b10100000); // Padded with zeros
    }

    #[test]
    fn test_pack_bits_all_zeros() {
        let bits = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let packed = NistWrapper::pack_bits_to_bytes(&bits);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0);
    }

    #[test]
    fn test_pack_bits_all_ones() {
        let bits = vec![1, 1, 1, 1, 1, 1, 1, 1];
        let packed = NistWrapper::pack_bits_to_bytes(&bits);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0xFF);
    }
}
