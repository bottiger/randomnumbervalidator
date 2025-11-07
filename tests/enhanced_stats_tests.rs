// Tests for enhanced_stats.rs
use randomnumbervalidator::enhanced_stats::{frequency_test, runs_test, longest_run_test, run_enhanced_tests};

#[test]
fn test_frequency_test_balanced() {
    let bits = vec![0, 1, 0, 1, 0, 1, 0, 1];
    let result = frequency_test(&bits);
    assert!(result.passed);
}

#[test]
fn test_frequency_test_unbalanced() {
    let bits = vec![1, 1, 1, 1, 1, 1, 1, 1];
    let result = frequency_test(&bits);
    assert!(!result.passed);
}

#[test]
fn test_runs_test() {
    let bits = vec![0, 1, 0, 1, 1, 0, 0, 1];
    let result = runs_test(&bits);
    assert!(result.statistic >= 0.0);
}

#[test]
fn test_longest_run_test() {
    let bits = vec![0, 0, 0, 0, 1, 1, 1, 0];
    let result = longest_run_test(&bits);
    assert_eq!(result.statistic, 4.0);
}

#[test]
fn test_enhanced_tests() {
    let bits = vec![0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1];
    let summary = run_enhanced_tests(&bits);
    assert!(summary.contains("Enhanced Statistical Analysis"));
    assert!(summary.contains("Tests Run"));
}
