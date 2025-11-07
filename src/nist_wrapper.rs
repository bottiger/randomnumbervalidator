use nistrs::prelude::*;
use std::collections::HashMap;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

use crate::nist_tests;
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
            warn!(
                "Insufficient bits for NIST tests: {} < {}",
                bits.len(),
                TestTier::TIER_1.min_bits
            );
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

        info!(
            "Running NIST tests on {} bits (Tier {}: {})",
            bits.len(),
            tier.level,
            tier.name
        );

        // Convert Vec<u8> (0s and 1s) to packed bytes for nistrs
        let packed_bytes = Self::pack_bits_to_bytes(bits);
        let bits_data = BitsData::from_binary(packed_bytes);

        // Run tests appropriate for this tier
        let test_results = Self::run_all_tests(&bits_data, &tier);

        // Parse results into structured format
        self.parse_test_results(bits, test_results, &tier)
    }

    /// Convert Vec<u8> where each element is 0 or 1 into packed bytes
    pub fn pack_bits_to_bytes(bits: &[u8]) -> Vec<u8> {
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
        let bit_count = data.len();

        // Get all test definitions and filter by tier and bit requirements
        for test_def in nist_tests::get_all_tests() {
            if test_def.should_run(tier.level, bit_count) {
                let test_results = (test_def.execute)(data);
                if !test_results.is_empty() {
                    results.insert(test_def.name.to_string(), test_results);
                }
            }
        }

        results
    }

    /// Calculate quality score from individual test results
    /// Returns (success_count, total_tests, avg_p_value, success_rate)
    ///
    /// The quality score combines two factors:
    /// 1. Pass rate (80% weight): Percentage of tests that pass (p >= 0.01)
    /// 2. P-value distribution (20% weight): How well p-values are distributed
    ///
    /// For truly random data:
    /// - All (or nearly all) tests should pass
    /// - P-values should average around 0.5
    /// - P-values all near 1.0 are suspicious (too uniform)
    /// - P-values all near 0.01 are marginal (barely passing)
    fn calculate_quality_score(individual_tests: &[NistTestResult]) -> (usize, usize, f64, f64) {
        let total_tests = individual_tests.len();
        let success_count = individual_tests.iter().filter(|t| t.passed).count();

        // Calculate average p-value for informational purposes
        let sum_p_values: f64 = individual_tests.iter().map(|t| t.p_value).sum();
        let avg_p_value = if total_tests > 0 {
            sum_p_values / total_tests as f64
        } else {
            0.0
        };

        if total_tests == 0 {
            return (0, 0, 0.0, 0.0);
        }

        // Component 1: Pass rate (percentage of tests with p >= 0.01)
        let pass_rate = (success_count as f64 / total_tests as f64) * 100.0;

        // Component 2: P-value distribution quality
        // For truly random data, p-values should be uniformly distributed (avg ~0.5)
        // Calculate average of passing tests only
        let passing_p_values: Vec<f64> = individual_tests
            .iter()
            .filter(|t| t.passed)
            .map(|t| t.p_value)
            .collect();

        let p_value_quality = if !passing_p_values.is_empty() {
            let avg_passing_p =
                passing_p_values.iter().sum::<f64>() / passing_p_values.len() as f64;

            // Ideal p-value average is 0.5 (uniform distribution)
            // Apply penalty for deviation from this ideal:
            // - P-values near 1.0: suspicious (too uniform), reduced quality
            // - P-values near 0.01: marginal (barely passing), reduced quality
            // - P-values near 0.5: ideal, full quality

            // Distance from ideal (0.5)
            let deviation = (avg_passing_p - 0.5).abs();

            // Convert to quality score (0-100)
            // Maximum deviation is 0.5 (from 0.0 or 1.0 to 0.5)
            // Linear penalty: 100% at deviation=0, 0% at deviation=0.5
            let quality = 100.0 * (1.0 - (deviation / 0.5));

            quality.max(0.0).min(100.0)
        } else {
            // If no tests pass, p-value quality is 0
            0.0
        };

        // Final quality score: weighted combination
        // Pass rate is dominant (80%), p-value distribution is secondary (20%)
        let success_rate = pass_rate * 0.8 + p_value_quality * 0.2;

        (success_count, total_tests, avg_p_value, success_rate)
    }

    /// Parse test results into structured format
    fn parse_test_results(
        &self,
        bits: &[u8],
        test_results: HashMap<String, Vec<TestResultT>>,
        tier: &TestTier,
    ) -> Result<NistResults, String> {
        let bit_count = bits.len();

        // Build individual test results
        let mut individual_tests = Vec::new();
        for (test_name, results_vec) in &test_results {
            // Each test may have multiple sub-results
            for (i, (passed, p_value)) in results_vec.iter().enumerate() {
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

        // Calculate quality score
        let (success_count, total_tests, avg_p_value, success_rate) =
            Self::calculate_quality_score(&individual_tests);

        // Generate raw output text with tier information
        let raw_output = Self::generate_raw_output(
            bit_count,
            &individual_tests,
            success_count,
            total_tests,
            success_rate,
            tier,
        );

        info!(
            "NIST tests completed (Tier {}): {}/{} passed, quality score: {:.1}% (avg p-value: {:.4})",
            tier.level, success_count, total_tests, success_rate, avg_p_value
        );

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
             Overall: {}/{} tests passed (binary pass/fail)\n\
             Quality Score: {:.1}% (weighted by p-values)\n\n\
             Individual Test Results:\n\
             ------------------------\n",
            bit_count,
            tier.level,
            tier.name,
            tier.description,
            success_count,
            total_tests,
            success_rate
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
                 Recommended: {} bits (~{} numbers) for optimal reliability\n\
                 Next Tier: Level {} ({}) requires {} bits (~{} numbers)\n",
                tier.level,
                tier.description,
                total_tests,
                tier.recommended_bits,
                tier.recommended_bits / 32,
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
