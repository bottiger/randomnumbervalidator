use randomnumbervalidator::nist_wrapper::NistWrapper;

fn main() {
    // Test with 160 bits (5 numbers)
    let bits_160: Vec<u8> = vec![0; 160];

    println!("Testing NIST with 160 bits...");
    let wrapper = NistWrapper::new();
    match wrapper.run_tests(&bits_160) {
        Ok(results) => {
            println!("Results:\n{}", results);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n\n========================================\n");

    // Test with 1000 bits (should give better results)
    let bits_1000: Vec<u8> = vec![0; 1000];

    println!("Testing NIST with 1000 bits...");
    match wrapper.run_tests(&bits_1000) {
        Ok(results) => {
            println!("Results:\n{}", results);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
