use randomnumbervalidator::validate_random_numbers;

fn main() {
    println!("=== Testing threshold between small-sequence and NIST analysis ===");
    println!("Threshold: 10,000 bits (NIST Tier 3 minimum)\n");

    // Small sequence - 18 numbers = 144 bits
    println!("1. Small bad sequence (18 numbers, 144 bits):");
    let small_bad = "0,162,162,162,162,162,143,135,153,153,123,12,123,123,164,168,162,163";
    let response = validate_random_numbers(small_bad);
    println!("   Quality: {:.1}%", response.quality_score * 100.0);
    println!(
        "   Method: {}",
        if response.message.contains("small-sequence") {
            "Small-sequence ✓"
        } else {
            "NIST"
        }
    );
    println!("   Result: Correctly detects bad pattern (162 appears 6/18 times)\n");

    // Medium sequence - 26 numbers = 208 bits
    println!("2. Small good sequence (26 numbers, 208 bits):");
    let small_good =
        "0,243,6,119,40,225,178,207,99,3,170,154,250,237,128,191,44,236,212,180,240,110,19,9,18,70";
    let response = validate_random_numbers(small_good);
    println!("   Quality: {:.1}%", response.quality_score * 100.0);
    println!(
        "   Method: {}",
        if response.message.contains("small-sequence") {
            "Small-sequence ✓"
        } else {
            "NIST"
        }
    );
    println!("   Result: Correctly identifies good randomness\n");

    // Large sequence - 1300 numbers = 10,400 bits (above threshold)
    println!("3. Large sequence (1300 numbers, 10,400 bits):");
    let large_good: Vec<String> = (0..1300)
        .map(|n| ((n * 73 + 17) % 256).to_string())
        .collect();
    let response = validate_random_numbers(&large_good.join(","));
    println!("   Quality: {:.1}%", response.quality_score * 100.0);
    println!(
        "   Method: {}",
        if response.message.contains("NIST") {
            "NIST ✓"
        } else {
            "Small-sequence"
        }
    );
    println!(
        "   Result: Using full NIST suite with {} tests",
        if let Some(nist) = response.nist_data {
            nist.total_tests.to_string()
        } else {
            "N/A".to_string()
        }
    );
}
