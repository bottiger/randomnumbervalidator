use randomnumbervalidator::nist_wrapper::NistWrapper;

fn main() {
    println!("Testing NIST Wrapper Path Resolution");
    println!("=====================================\n");

    let wrapper = NistWrapper::new();

    println!("Current working directory: {:?}", std::env::current_dir());

    // Test with some sample bits
    let test_numbers = "42,17,89,3,56,91,23,67";
    println!("\nTesting with numbers: {}", test_numbers);

    // Convert to bits (simplified)
    let bits: Vec<u8> = test_numbers
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .flat_map(|num| {
            (0..32)
                .rev()
                .map(move |i| ((num >> i) & 1) as u8)
                .collect::<Vec<u8>>()
        })
        .collect();

    println!("Total bits: {}", bits.len());

    match wrapper.run_tests(&bits) {
        Ok(results) => {
            println!("\n✓ NIST Tests Completed Successfully!\n");
            println!("{:?}", results);
        }
        Err(e) => {
            println!("\n✗ NIST Tests Failed:\n{}", e);
        }
    }
}
