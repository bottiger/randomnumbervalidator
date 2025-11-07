use serde::{Deserialize, Serialize};
/// Enhanced statistical tests for small datasets
/// These tests work well with limited data where NIST tests cannot run
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticalTestResult {
    pub test_name: String,
    pub passed: bool,
    pub statistic: f64,
    pub p_value: Option<f64>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedTestResults {
    pub bit_count: usize,
    pub tests_run: usize,
    pub tests_passed: usize,
    pub pass_rate: f64,
    pub individual_tests: Vec<StatisticalTestResult>,
}

/// Run comprehensive statistical analysis and return structured data
pub fn run_enhanced_tests_structured(bits: &[u8]) -> EnhancedTestResults {
    // Run all tests
    let results = vec![
        frequency_test(bits),
        runs_test(bits),
        longest_run_test(bits),
        poker_test(bits),
        autocorrelation_test(bits),
        pattern_distribution_test(bits),
    ];

    // Calculate overall statistics
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let pass_rate = (passed_tests as f64 / total_tests as f64) * 100.0;

    EnhancedTestResults {
        bit_count: bits.len(),
        tests_run: total_tests,
        tests_passed: passed_tests,
        pass_rate,
        individual_tests: results,
    }
}

/// Run comprehensive statistical analysis on bit sequence (legacy string output)
pub fn run_enhanced_tests(bits: &[u8]) -> String {
    let structured = run_enhanced_tests_structured(bits);
    let results = &structured.individual_tests;
    let total_tests = structured.tests_run;
    let passed_tests = structured.tests_passed;
    let pass_rate = structured.pass_rate;

    // Generate summary
    let mut summary = format!(
        "Enhanced Statistical Analysis (Small Dataset)\n\
         ===============================================\n\
         Input Size: {} bits\n\
         Tests Run: {}\n\
         Tests Passed: {}/{} ({:.1}%)\n\n\
         Individual Test Results:\n\
         -----------------------\n",
        bits.len(),
        total_tests,
        passed_tests,
        total_tests,
        pass_rate
    );

    for result in results {
        let status = if result.passed {
            "PASS ✓"
        } else {
            "FAIL ✗"
        };
        let p_val_str = result
            .p_value
            .map(|p| format!("p={:.4}", p))
            .unwrap_or_else(|| format!("stat={:.4}", result.statistic));

        summary.push_str(&format!(
            "{:30} {} ({})\n  {}\n\n",
            result.test_name, status, p_val_str, result.description
        ));
    }

    // Add interpretation
    summary.push_str("\nInterpretation:\n");
    summary.push_str("---------------\n");
    if pass_rate >= 80.0 {
        summary.push_str("✓ GOOD: The sequence shows good randomness properties.\n");
    } else if pass_rate >= 50.0 {
        summary.push_str("⚠ MODERATE: The sequence shows some randomness but has weaknesses.\n");
    } else {
        summary.push_str("✗ POOR: The sequence shows poor randomness properties.\n");
    }

    summary
        .push_str("\nNote: These are simplified statistical tests suitable for small datasets.\n");
    summary.push_str("For comprehensive analysis, provide 313+ numbers (10,000+ bits) to enable full NIST testing.\n");

    summary
}

/// Frequency (Monobit) Test
/// Tests if the number of 1s and 0s are approximately equal
pub fn frequency_test(bits: &[u8]) -> StatisticalTestResult {
    let n = bits.len() as f64;
    let ones = bits.iter().filter(|&&b| b == 1).count() as f64;
    let zeros = (bits.len() - ones as usize) as f64;

    // Calculate test statistic: |ones - zeros| / sqrt(n)
    let statistic = ((ones - zeros).abs()) / n.sqrt();

    // Approximate p-value using normal distribution
    // For a good sequence, statistic should be < 2.0
    let passed = statistic < 2.0;
    let description = format!(
        "Ones: {:.0}, Zeros: {:.0}, Ratio: {:.3} (expect ~0.500)",
        ones,
        zeros,
        ones / n
    );

    StatisticalTestResult {
        test_name: "Frequency Test".to_string(),
        passed,
        statistic,
        p_value: None,
        description,
    }
}

/// Runs Test
/// Tests for oscillation between 0 and 1
pub fn runs_test(bits: &[u8]) -> StatisticalTestResult {
    if bits.len() < 2 {
        return StatisticalTestResult {
            test_name: "Runs Test".to_string(),
            passed: false,
            statistic: 0.0,
            p_value: None,
            description: "Insufficient data".to_string(),
        };
    }

    let n = bits.len() as f64;
    let ones = bits.iter().filter(|&&b| b == 1).count() as f64;
    let prop = ones / n;

    // Count runs (sequences of consecutive identical bits)
    let mut runs = 1;
    for i in 1..bits.len() {
        if bits[i] != bits[i - 1] {
            runs += 1;
        }
    }

    // Expected runs for random sequence
    let expected_runs = 2.0 * n * prop * (1.0 - prop) + 1.0;
    let variance = 2.0 * n * prop * (1.0 - prop) * (2.0 * n * prop * (1.0 - prop) - n);
    let std_dev = variance.sqrt();

    // Calculate test statistic
    let statistic = if std_dev > 0.0 {
        ((runs as f64) - expected_runs).abs() / std_dev
    } else {
        0.0
    };

    let passed = statistic < 2.0;
    let description = format!(
        "Observed runs: {}, Expected: {:.1}, Statistic: {:.3}",
        runs, expected_runs, statistic
    );

    StatisticalTestResult {
        test_name: "Runs Test".to_string(),
        passed,
        statistic,
        p_value: None,
        description,
    }
}

/// Longest Run Test
/// Tests the length of longest run of 1s or 0s
pub fn longest_run_test(bits: &[u8]) -> StatisticalTestResult {
    if bits.is_empty() {
        return StatisticalTestResult {
            test_name: "Longest Run Test".to_string(),
            passed: false,
            statistic: 0.0,
            p_value: None,
            description: "No data".to_string(),
        };
    }

    let mut longest_run = 1;
    let mut current_run = 1;
    let mut current_bit = bits[0];

    for &bit in bits.iter().skip(1) {
        if bit == current_bit {
            current_run += 1;
            longest_run = longest_run.max(current_run);
        } else {
            current_run = 1;
            current_bit = bit;
        }
    }

    // For random data, longest run should be approximately log2(n)
    let n = bits.len() as f64;
    let expected_longest = (n.log2() * 1.5).ceil() as usize;

    let passed = longest_run <= expected_longest * 2;
    let description = format!(
        "Longest run: {}, Expected: ~{}, {}",
        longest_run,
        expected_longest,
        if passed {
            "within normal range"
        } else {
            "suspiciously long"
        }
    );

    StatisticalTestResult {
        test_name: "Longest Run Test".to_string(),
        passed,
        statistic: longest_run as f64,
        p_value: None,
        description,
    }
}

/// Poker Test
/// Tests distribution of bit patterns
fn poker_test(bits: &[u8]) -> StatisticalTestResult {
    if bits.len() < 4 {
        return StatisticalTestResult {
            test_name: "Poker Test".to_string(),
            passed: false,
            statistic: 0.0,
            p_value: None,
            description: "Insufficient data (need at least 4 bits)".to_string(),
        };
    }

    // Count 4-bit patterns
    let mut pattern_counts: HashMap<u8, usize> = HashMap::new();
    let block_size = 4;
    let num_blocks = bits.len() / block_size;

    for i in 0..num_blocks {
        let start = i * block_size;
        let mut pattern = 0u8;
        for j in 0..block_size {
            pattern = (pattern << 1) | bits[start + j];
        }
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }

    // Calculate chi-square statistic
    let expected_count = num_blocks as f64 / 16.0; // 16 possible 4-bit patterns
    let mut chi_square = 0.0;

    for &count in pattern_counts.values() {
        let diff = count as f64 - expected_count;
        chi_square += (diff * diff) / expected_count;
    }

    // For 15 degrees of freedom, chi-square should be < 25 (roughly)
    let passed = chi_square < 25.0 && num_blocks >= 4;
    let description = format!(
        "Patterns found: {}/16, Chi-square: {:.2}, {} blocks analyzed",
        pattern_counts.len(),
        chi_square,
        num_blocks
    );

    StatisticalTestResult {
        test_name: "Poker Test (Pattern Distribution)".to_string(),
        passed,
        statistic: chi_square,
        p_value: None,
        description,
    }
}

/// Autocorrelation Test
/// Tests for correlations between bits at different positions
fn autocorrelation_test(bits: &[u8]) -> StatisticalTestResult {
    if bits.len() < 10 {
        return StatisticalTestResult {
            test_name: "Autocorrelation Test".to_string(),
            passed: false,
            statistic: 0.0,
            p_value: None,
            description: "Insufficient data (need at least 10 bits)".to_string(),
        };
    }

    // Test autocorrelation at lag 1 and lag 2
    let mut max_correlation: f64 = 0.0;
    let n = bits.len();

    for lag in 1..=2.min(n / 4) {
        let mut matches = 0;
        for i in 0..n - lag {
            if bits[i] == bits[i + lag] {
                matches += 1;
            }
        }

        let correlation = (matches as f64) / ((n - lag) as f64);
        max_correlation = max_correlation.max((correlation - 0.5).abs());
    }

    // For random data, correlation should be close to 0.5
    let statistic = max_correlation;
    let passed = statistic < 0.15; // Allow 15% deviation

    let description = format!(
        "Max autocorrelation deviation: {:.3} (expect < 0.15 for randomness)",
        statistic
    );

    StatisticalTestResult {
        test_name: "Autocorrelation Test".to_string(),
        passed,
        statistic,
        p_value: None,
        description,
    }
}

/// Pattern Distribution Test
/// Tests for common non-random patterns
fn pattern_distribution_test(bits: &[u8]) -> StatisticalTestResult {
    if bits.len() < 8 {
        return StatisticalTestResult {
            test_name: "Pattern Distribution Test".to_string(),
            passed: false,
            statistic: 0.0,
            p_value: None,
            description: "Insufficient data".to_string(),
        };
    }

    // Check for obvious patterns
    let mut issues = Vec::new();

    // Check for long sequences of same bit
    let max_same = find_max_consecutive_same(bits);
    if max_same > (bits.len() / 4).max(8) {
        issues.push(format!("{} consecutive identical bits", max_same));
    }

    // Check for alternating pattern (010101...)
    let alternating_count = count_alternating_pattern(bits);
    let alternating_ratio = alternating_count as f64 / bits.len() as f64;
    if alternating_ratio > 0.9 {
        issues.push(format!(
            "{:.0}% alternating pattern",
            alternating_ratio * 100.0
        ));
    }

    // Check for repeating blocks
    if has_repeating_blocks(bits) {
        issues.push("Repeating block pattern detected".to_string());
    }

    let passed = issues.is_empty();
    let description = if passed {
        "No obvious non-random patterns detected".to_string()
    } else {
        format!("Issues found: {}", issues.join("; "))
    };

    StatisticalTestResult {
        test_name: "Pattern Distribution Test".to_string(),
        passed,
        statistic: issues.len() as f64,
        p_value: None,
        description,
    }
}

fn find_max_consecutive_same(bits: &[u8]) -> usize {
    if bits.is_empty() {
        return 0;
    }

    let mut max_count = 1;
    let mut current_count = 1;

    for i in 1..bits.len() {
        if bits[i] == bits[i - 1] {
            current_count += 1;
            max_count = max_count.max(current_count);
        } else {
            current_count = 1;
        }
    }

    max_count
}

fn count_alternating_pattern(bits: &[u8]) -> usize {
    if bits.len() < 2 {
        return 0;
    }

    let mut count = 0;
    for i in 1..bits.len() {
        if bits[i] != bits[i - 1] {
            count += 1;
        }
    }
    count
}

fn has_repeating_blocks(bits: &[u8]) -> bool {
    if bits.len() < 16 {
        return false;
    }

    // Check for repeating 8-bit blocks
    let block_size = 8;
    for i in 0..bits.len().saturating_sub(block_size * 2) {
        let block1 = &bits[i..i + block_size];
        let block2 = &bits[i + block_size..i + block_size * 2];

        if block1 == block2 {
            // Found a repeating block, check if it repeats more
            return true;
        }
    }

    false
}

