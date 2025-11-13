use randomnumbervalidator::validate_random_numbers;

fn main() {
    let input =
        "0,243,6,119,40,225,178,207,99,3,170,154,250,237,128,191,44,236,212,180,240,110,19,9,18,70";
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
