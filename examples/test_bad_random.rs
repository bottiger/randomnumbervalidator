use randomnumbervalidator::validate_random_numbers;

fn main() {
    let input = "0,162,162,162,162,162,143,135,153,153,123,12,123,123,164,168,162,163";
    let response = validate_random_numbers(input);

    println!("Input: {}", input);
    println!("Valid: {}", response.valid);
    println!("Quality Score: {:.2}%", response.quality_score * 100.0);
    println!("Message: {}", response.message);

    if let Some(nist_data) = &response.nist_data {
        println!("\nNIST Results:");
        println!(
            "  Tests passed: {}/{}",
            nist_data.tests_passed, nist_data.total_tests
        );
        println!("  Success rate: {:.2}%", nist_data.success_rate);
        println!("  Bit count: {}", nist_data.bit_count);
    }
}
