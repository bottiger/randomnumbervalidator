/// Statistical tests for small sequences of numbers
/// These tests work on the number level to detect obvious patterns
/// that NIST tests miss due to insufficient data
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SmallSequenceTestResult {
    pub name: String,
    pub passed: bool,
    pub details: Option<String>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SmallSequenceResults {
    pub number_count: usize,
    pub unique_count: usize,
    pub quality_score: f64,
    pub tests_passed: usize,
    pub total_tests: usize,
    pub issues: Vec<String>,
    pub individual_tests: Vec<SmallSequenceTestResult>,
}

/// Analyze a small sequence of numbers for randomness quality
pub fn analyze_small_sequence(numbers: &[u32]) -> SmallSequenceResults {
    if numbers.is_empty() {
        return SmallSequenceResults {
            number_count: 0,
            unique_count: 0,
            quality_score: 0.0,
            tests_passed: 0,
            total_tests: 0,
            issues: vec!["No numbers provided".to_string()],
            individual_tests: vec![],
        };
    }

    let mut issues = Vec::new();
    let mut individual_tests = Vec::new();
    let mut tests_passed = 0;
    let total_tests = 5;

    // Calculate basic statistics
    let number_count = numbers.len();
    let unique_count = numbers
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    let uniqueness_ratio = unique_count as f64 / number_count as f64;

    // Test 1: Frequency analysis - detect excessive repetition
    let mut freq_issue = None;
    let (freq_passed, freq_p_value) = test_frequency_distribution(numbers, &mut freq_issue);
    if freq_passed {
        tests_passed += 1;
    } else if let Some(issue) = &freq_issue {
        issues.push(issue.clone());
    }
    individual_tests.push(SmallSequenceTestResult {
        name: "Frequency Analysis".to_string(),
        passed: freq_passed,
        details: freq_issue,
        p_value: Some(freq_p_value),
    });

    // Test 2: Uniqueness ratio - detect too many duplicates
    let mut uniqueness_issue = None;
    let (uniqueness_passed, uniqueness_p_value) =
        test_uniqueness(number_count, unique_count, &mut uniqueness_issue);
    if uniqueness_passed {
        tests_passed += 1;
    } else if let Some(issue) = &uniqueness_issue {
        issues.push(issue.clone());
    }
    individual_tests.push(SmallSequenceTestResult {
        name: "Uniqueness Ratio".to_string(),
        passed: uniqueness_passed,
        details: uniqueness_issue,
        p_value: Some(uniqueness_p_value),
    });

    // Test 3: Chi-squared test for uniformity
    let mut chi_issue = None;
    let (chi_passed, chi_p_value) = test_chi_squared(numbers, &mut chi_issue);
    if chi_passed {
        tests_passed += 1;
    } else if let Some(issue) = &chi_issue {
        issues.push(issue.clone());
    }
    individual_tests.push(SmallSequenceTestResult {
        name: "Chi-Squared Uniformity".to_string(),
        passed: chi_passed,
        details: chi_issue,
        p_value: Some(chi_p_value),
    });

    // Test 4: Serial correlation
    let mut serial_issue = None;
    let (serial_passed, serial_p_value) = test_serial_correlation(numbers, &mut serial_issue);
    if serial_passed {
        tests_passed += 1;
    } else if let Some(issue) = &serial_issue {
        issues.push(issue.clone());
    }
    individual_tests.push(SmallSequenceTestResult {
        name: "Serial Correlation".to_string(),
        passed: serial_passed,
        details: serial_issue,
        p_value: Some(serial_p_value),
    });

    // Test 5: Sequential pattern detection
    let mut sequential_issue = None;
    let (sequential_passed, sequential_p_value) =
        test_sequential_pattern(numbers, &mut sequential_issue);
    if sequential_passed {
        tests_passed += 1;
    } else if let Some(issue) = &sequential_issue {
        issues.push(issue.clone());
    }
    individual_tests.push(SmallSequenceTestResult {
        name: "Sequential Pattern Detection".to_string(),
        passed: sequential_passed,
        details: sequential_issue,
        p_value: Some(sequential_p_value),
    });

    // Calculate quality score
    let pass_rate = tests_passed as f64 / total_tests as f64;

    // Apply penalties for severe issues
    let mut quality_score = pass_rate;

    // Heavy penalty for sequential patterns (they're very non-random)
    if issues
        .iter()
        .any(|s| s.contains("Sequential") || s.contains("sequential"))
    {
        quality_score *= 0.3; // Severe penalty for sequential patterns
    }

    // Heavy penalty if uniqueness is very low
    if uniqueness_ratio < 0.7 {
        quality_score *= 0.6;
    }

    if uniqueness_ratio < 0.5 {
        quality_score *= 0.5;
    }

    // Additional penalty for very small unique ratio
    if uniqueness_ratio < 0.3 {
        quality_score *= 0.4;
    }

    SmallSequenceResults {
        number_count,
        unique_count,
        quality_score,
        tests_passed,
        total_tests,
        issues,
        individual_tests,
    }
}

/// Test 1: Check if any number appears too frequently
fn test_frequency_distribution(numbers: &[u32], issue: &mut Option<String>) -> (bool, f64) {
    let mut frequency: HashMap<u32, usize> = HashMap::new();
    for &num in numbers {
        *frequency.entry(num).or_insert(0) += 1;
    }

    let n = numbers.len() as f64;
    let mut max_freq = 0;
    let mut max_freq_value = 0;

    for (&value, &count) in &frequency {
        if count > max_freq {
            max_freq = count;
            max_freq_value = value;
        }
    }

    let max_freq_ratio = max_freq as f64 / n;

    // Flag if any value appears more than expected by chance
    // For uniform distribution, each value should appear ~1/range times
    // For small sequences, we need stricter thresholds to catch bad patterns
    let threshold = if n < 10.0 {
        0.35 // 35% for very small sequences
    } else if n < 20.0 {
        0.25 // 25% for small sequences (18 numbers -> 162 appears 6 times = 33%)
    } else if n < 50.0 {
        0.18 // 18% for medium sequences
    } else {
        0.12 // 12% for larger sequences
    };

    // Calculate p-value: probability that max frequency occurred by chance
    // Using 1 - max_freq_ratio as a simple approximation
    let p_value = 1.0 - max_freq_ratio;

    if max_freq_ratio > threshold {
        *issue = Some(format!(
            "Value {} appears {} times ({:.1}% of sequence, expected < {:.1}%)",
            max_freq_value,
            max_freq,
            max_freq_ratio * 100.0,
            threshold * 100.0
        ));
        (false, p_value)
    } else {
        (true, p_value)
    }
}

/// Test 2: Check uniqueness ratio
fn test_uniqueness(total: usize, unique: usize, issue: &mut Option<String>) -> (bool, f64) {
    let ratio = unique as f64 / total as f64;

    // Expect at least 60% unique values for good randomness
    // (in small sequences, some repetition is expected but not too much)
    let threshold = if total < 10 {
        0.7 // 70% for very small
    } else if total < 20 {
        0.65 // 65% for small (18 numbers -> need at least 12 unique)
    } else {
        0.75 // 75% for larger
    };

    // P-value: how close we are to the threshold (ratio / threshold)
    let p_value = ratio / threshold;

    if ratio < threshold {
        *issue = Some(format!(
            "Only {} unique values out of {} ({:.1}%, expected >= {:.1}%)",
            unique,
            total,
            ratio * 100.0,
            threshold * 100.0
        ));
        (false, p_value)
    } else {
        (true, p_value)
    }
}

/// Test 3: Chi-squared test for uniform distribution
fn test_chi_squared(numbers: &[u32], issue: &mut Option<String>) -> (bool, f64) {
    if numbers.len() < 5 {
        return (true, 1.0); // Skip for very small sequences
    }

    // Count frequencies
    let mut frequency: HashMap<u32, usize> = HashMap::new();
    for &num in numbers {
        *frequency.entry(num).or_insert(0) += 1;
    }

    let k = frequency.len() as f64; // number of unique values
    let n = numbers.len() as f64; // total numbers
    let expected = n / k; // expected count per value

    // Calculate chi-squared statistic
    let mut chi_squared = 0.0;
    for &count in frequency.values() {
        let diff = count as f64 - expected;
        chi_squared += (diff * diff) / expected;
    }

    // For small sequences, use a relaxed threshold
    // Critical value depends on degrees of freedom (k-1)
    // Using a simple heuristic: chi-squared should be < k * 2
    let threshold = k * 2.0;

    // P-value: 1 - (chi_squared / (threshold * 2)) clamped to [0, 1]
    let p_value = (1.0 - (chi_squared / (threshold * 2.0))).clamp(0.0, 1.0);

    if chi_squared > threshold {
        *issue = Some(format!(
            "Non-uniform distribution (chi-squared = {:.2}, threshold = {:.2})",
            chi_squared, threshold
        ));
        (false, p_value)
    } else {
        (true, p_value)
    }
}

/// Test 4: Serial correlation test
fn test_serial_correlation(numbers: &[u32], issue: &mut Option<String>) -> (bool, f64) {
    if numbers.len() < 3 {
        return (true, 1.0); // Skip for very small sequences
    }

    // Normalize numbers to 0-1 range for correlation calculation
    let min = *numbers.iter().min().unwrap() as f64;
    let max = *numbers.iter().max().unwrap() as f64;
    let range = max - min;

    if range == 0.0 {
        *issue = Some("All numbers are identical".to_string());
        return (false, 0.0);
    }

    let normalized: Vec<f64> = numbers.iter().map(|&x| (x as f64 - min) / range).collect();

    // Calculate lag-1 autocorrelation
    let mean: f64 = normalized.iter().sum::<f64>() / normalized.len() as f64;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 0..normalized.len() - 1 {
        numerator += (normalized[i] - mean) * (normalized[i + 1] - mean);
    }

    for &x in &normalized {
        denominator += (x - mean) * (x - mean);
    }

    let correlation = if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    };

    // For random data, correlation should be close to 0
    // Allow more deviation for small sequences
    let threshold = if numbers.len() < 10 {
        0.6
    } else if numbers.len() < 20 {
        0.5
    } else {
        0.4
    };

    // P-value: 1 - (|correlation| / threshold) clamped to [0, 1]
    let p_value = (1.0 - (correlation.abs() / threshold)).clamp(0.0, 1.0);

    if correlation.abs() > threshold {
        *issue = Some(format!(
            "High serial correlation ({:.2}, expected < {:.2})",
            correlation.abs(),
            threshold
        ));
        (false, p_value)
    } else {
        (true, p_value)
    }
}

/// Test 5: Sequential pattern detection
fn test_sequential_pattern(numbers: &[u32], issue: &mut Option<String>) -> (bool, f64) {
    if numbers.len() < 3 {
        return (true, 1.0); // Skip for very small sequences
    }

    // Check for ascending sequences (0,1,2,3... or 10,11,12,13...)
    let mut ascending_count = 0;
    let mut descending_count = 0;
    let mut total_diffs = 0;

    for i in 1..numbers.len() {
        let diff = numbers[i] as i64 - numbers[i - 1] as i64;
        total_diffs += 1;

        if diff == 1 {
            ascending_count += 1;
        } else if diff == -1 {
            descending_count += 1;
        }
    }

    let ascending_ratio = ascending_count as f64 / total_diffs as f64;
    let descending_ratio = descending_count as f64 / total_diffs as f64;

    // If more than 70% of transitions are +1 or -1, it's likely sequential
    let threshold = 0.7;
    let max_ratio = ascending_ratio.max(descending_ratio);

    // P-value: 1 - (max_ratio / threshold) clamped to [0, 1]
    let p_value = (1.0 - (max_ratio / threshold)).clamp(0.0, 1.0);

    if ascending_ratio > threshold {
        *issue = Some(format!(
            "Sequential ascending pattern detected ({:.1}% of transitions are +1)",
            ascending_ratio * 100.0
        ));
        (false, p_value)
    } else if descending_ratio > threshold {
        *issue = Some(format!(
            "Sequential descending pattern detected ({:.1}% of transitions are -1)",
            descending_ratio * 100.0
        ));
        (false, p_value)
    } else {
        (true, p_value)
    }
}
