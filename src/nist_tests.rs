use nistrs::prelude::*;

/// Definition of a NIST test with its requirements and execution logic
pub struct NistTestDefinition {
    /// Human-readable name for the test
    pub name: &'static str,
    /// Minimum tier level required to run this test
    pub tier: u8,
    /// Minimum number of bits required (None = no specific requirement beyond tier)
    pub min_bits: Option<usize>,
    /// Function to execute the test
    /// Returns a vector of test results (most tests return 1, some return multiple)
    pub execute: fn(&BitsData) -> Vec<TestResultT>,
}

impl NistTestDefinition {
    /// Check if this test should run given the current tier and bit count
    pub fn should_run(&self, tier_level: u8, bit_count: usize) -> bool {
        if tier_level < self.tier {
            return false;
        }
        if let Some(min) = self.min_bits {
            if bit_count < min {
                return false;
            }
        }
        true
    }
}

/// Get all NIST test definitions
/// This is the single source of truth for all available tests and their requirements
pub fn get_all_tests() -> Vec<NistTestDefinition> {
    vec![
        // === TIER 1: Basic tests (100+ bits) ===
        NistTestDefinition {
            name: "Frequency",
            tier: 1,
            min_bits: None,
            execute: |data| vec![frequency_test(data)],
        },
        NistTestDefinition {
            name: "Runs",
            tier: 1,
            min_bits: None,
            execute: |data| vec![runs_test(data)],
        },
        NistTestDefinition {
            name: "FFT",
            tier: 1,
            min_bits: None,
            execute: |data| vec![fft_test(data)],
        },
        NistTestDefinition {
            name: "Universal",
            tier: 1,
            min_bits: Some(1000), // Requires 1000+ bits to avoid overflow
            execute: |data| vec![universal_test(data)],
        },
        NistTestDefinition {
            name: "CumulativeSums-Forward",
            tier: 1,
            min_bits: None,
            execute: |data| {
                let results = cumulative_sums_test(data);
                vec![results[0]]
            },
        },
        NistTestDefinition {
            name: "CumulativeSums-Reverse",
            tier: 1,
            min_bits: None,
            execute: |data| {
                let results = cumulative_sums_test(data);
                vec![results[1]]
            },
        },
        // === TIER 2: Block and template tests (1,000+ bits) ===
        NistTestDefinition {
            name: "BlockFrequency",
            tier: 2,
            min_bits: None,
            execute: |data| {
                let block_size = if data.len() >= 1000 {
                    100
                } else {
                    data.len() / 10
                };
                if block_size > 0 {
                    if let Ok(result) = block_frequency_test(data, block_size) {
                        return vec![result];
                    }
                }
                vec![]
            },
        },
        NistTestDefinition {
            name: "NonOverlappingTemplate",
            tier: 2,
            min_bits: None,
            execute: |data| {
                let template_size = 9;
                non_overlapping_template_test(data, template_size).unwrap_or_default()
            },
        },
        NistTestDefinition {
            name: "OverlappingTemplate",
            tier: 2,
            min_bits: None,
            execute: |data| {
                let template_size = 9;
                vec![overlapping_template_test(data, template_size)]
            },
        },
        // === TIER 3: Complex tests (10,000+ bits) ===
        NistTestDefinition {
            name: "LongestRun",
            tier: 3,
            min_bits: None,
            execute: |data| {
                if let Ok(result) = longest_run_of_ones_test(data) {
                    vec![result]
                } else {
                    vec![]
                }
            },
        },
        NistTestDefinition {
            name: "Rank",
            tier: 3,
            min_bits: None,
            execute: |data| {
                if let Ok(result) = rank_test(data) {
                    vec![result]
                } else {
                    vec![]
                }
            },
        },
        NistTestDefinition {
            name: "ApproximateEntropy",
            tier: 3,
            min_bits: Some(200), // Need at least m=2
            execute: |data| {
                let m_param = 10.min(data.len() / 100);
                if m_param >= 2 {
                    vec![approximate_entropy_test(data, m_param)]
                } else {
                    vec![]
                }
            },
        },
        NistTestDefinition {
            name: "Serial-1",
            tier: 3,
            min_bits: Some(200), // Need at least m=2
            execute: |data| {
                let serial_m = 16.min(data.len() / 100);
                if serial_m >= 2 {
                    let results = serial_test(data, serial_m);
                    vec![results[0]]
                } else {
                    vec![]
                }
            },
        },
        NistTestDefinition {
            name: "Serial-2",
            tier: 3,
            min_bits: Some(200), // Need at least m=2
            execute: |data| {
                let serial_m = 16.min(data.len() / 100);
                if serial_m >= 2 {
                    let results = serial_test(data, serial_m);
                    vec![results[1]]
                } else {
                    vec![]
                }
            },
        },
        // === TIER 4: Heavy tests (100,000+ bits) ===
        NistTestDefinition {
            name: "RandomExcursions",
            tier: 4,
            min_bits: None,
            execute: |data| {
                random_excursions_test(data)
                    .map(|results| results.to_vec())
                    .unwrap_or_default()
            },
        },
        NistTestDefinition {
            name: "RandomExcursionsVariant",
            tier: 4,
            min_bits: None,
            execute: |data| {
                random_excursions_variant_test(data)
                    .map(|results| results.to_vec())
                    .unwrap_or_default()
            },
        },
        NistTestDefinition {
            name: "LinearComplexity",
            tier: 4,
            min_bits: Some(10000), // Need block_size >= 100
            execute: |data| {
                let lc_block_size = 500.min(data.len() / 100);
                if lc_block_size >= 100 {
                    vec![linear_complexity_test(data, lc_block_size)]
                } else {
                    vec![]
                }
            },
        },
    ]
}
