// Tests for nist_wrapper.rs
use randomnumbervalidator::nist_wrapper::NistWrapper;

#[test]
fn test_nist_wrapper_creation() {
    let _wrapper = NistWrapper::new();
    // Verify we can create the wrapper
}

#[test]
fn test_run_tests_insufficient_bits() {
    let wrapper = NistWrapper::new();
    let bits = vec![1, 0, 1, 0]; // Only 4 bits, need at least 100,000

    let result = wrapper.run_tests(&bits);
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("100000") || error_msg.contains("require"));
}

#[test]
fn test_nist_wrapper_default() {
    let _wrapper = NistWrapper::default();
    // Verify we can create the wrapper with default
}

#[test]
fn test_pack_bits_to_bytes() {
    // Test: 8 bits should pack into 1 byte
    let bits = vec![1, 0, 1, 0, 1, 0, 1, 0];
    let packed = NistWrapper::pack_bits_to_bytes(&bits);
    assert_eq!(packed.len(), 1);
    assert_eq!(packed[0], 0b10101010);
}

#[test]
fn test_pack_bits_to_bytes_partial() {
    // Test: 12 bits should pack into 2 bytes (last byte padded)
    let bits = vec![1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0];
    let packed = NistWrapper::pack_bits_to_bytes(&bits);
    assert_eq!(packed.len(), 2);
    assert_eq!(packed[0], 0b11110000);
    assert_eq!(packed[1], 0b10100000); // Padded with zeros
}

#[test]
fn test_pack_bits_all_zeros() {
    let bits = vec![0, 0, 0, 0, 0, 0, 0, 0];
    let packed = NistWrapper::pack_bits_to_bytes(&bits);
    assert_eq!(packed.len(), 1);
    assert_eq!(packed[0], 0);
}

#[test]
fn test_pack_bits_all_ones() {
    let bits = vec![1, 1, 1, 1, 1, 1, 1, 1];
    let packed = NistWrapper::pack_bits_to_bytes(&bits);
    assert_eq!(packed.len(), 1);
    assert_eq!(packed[0], 0xFF);
}
