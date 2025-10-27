use randomnumbervalidator::{validate_random_numbers, prepare_input_for_nist};

#[test]
fn test_integration_basic_validation() {
    let input = "123,456,789";
    let response = validate_random_numbers(input);

    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
    assert!(!response.message.is_empty());
}

#[test]
fn test_integration_empty_input() {
    let input = "";
    let response = validate_random_numbers(input);

    assert!(!response.valid);
    assert_eq!(response.quality_score, 0.0);
}

#[test]
fn test_integration_large_sequence() {
    let input = "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20";
    let response = validate_random_numbers(input);

    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
}

#[test]
fn test_prepare_input_format() {
    let result = prepare_input_for_nist("42,17");
    assert!(result.is_ok());

    let bits = result.unwrap();
    // 2 numbers * 32 bits = 64 bits
    assert_eq!(bits.len(), 64);

    // Check that all bits are either 0 or 1
    for bit in bits {
        assert!(bit == 0 || bit == 1);
    }
}
