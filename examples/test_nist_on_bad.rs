use randomnumbervalidator::nist_wrapper::NistWrapper;
use randomnumbervalidator::prepare_input_for_nist;

fn main() {
    // The bad repeating sequence - 162 appears 6 times out of 18
    let input = "0,162,162,162,162,162,143,135,153,153,123,12,123,123,164,168,162,163";

    let bits = prepare_input_for_nist(input).expect("Failed to parse");

    println!("Testing bad repeating pattern with NIST:");
    println!("Input: {}", input);
    println!("Bits: {} bits", bits.len());
    println!();

    let wrapper = NistWrapper::new();
    match wrapper.run_tests(&bits) {
        Ok(results) => {
            println!("NIST Results:");
            println!(
                "  Tests passed: {}/{}",
                results.tests_passed, results.total_tests
            );
            println!("  Success rate: {:.2}%", results.success_rate);
            println!();
            println!("Individual tests:");
            for test in &results.individual_tests {
                println!(
                    "  {} {}: p-value = {:.6}",
                    if test.passed { "✓" } else { "✗" },
                    test.name,
                    test.p_value
                );
            }
        }
        Err(e) => {
            println!("NIST Error: {}", e);
        }
    }
}
