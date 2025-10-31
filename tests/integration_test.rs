use randomnumbervalidator::{
    prepare_input_for_nist, validate_random_numbers, validate_random_numbers_with_nist,
    ValidationRequest,
};

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

#[test]
fn test_integration_malformed_input() {
    let inputs = vec!["abc,def", "hello world", "1,2,abc,3", "test123"];

    for input in inputs {
        let response = validate_random_numbers(input);
        assert!(!response.valid, "Input '{}' should be invalid", input);
        assert_eq!(
            response.quality_score, 0.0,
            "Input '{}' should have 0 quality score",
            input
        );
    }
}

#[test]
fn test_integration_various_delimiters() {
    let inputs = vec![
        ("1,2,3", "comma"),
        ("1 2 3", "space"),
        ("1\n2\n3", "newline"),
        ("1\t2\t3", "tab"),
        ("1;2;3", "semicolon"),
        ("1|2|3", "pipe"),
    ];

    for (input, delimiter_name) in inputs {
        let response = validate_random_numbers(input);
        assert!(
            response.quality_score >= 0.0 && response.quality_score <= 1.0,
            "Failed with {} delimiter",
            delimiter_name
        );
    }
}

#[test]
fn test_integration_boundary_numbers() {
    // Test with boundary values
    let inputs = vec![
        ("0", "zero"),
        ("1", "one"),
        ("4294967295", "u32_max"), // u32::MAX
    ];

    for (input, description) in inputs {
        let response = validate_random_numbers(input);
        assert!(
            response.quality_score >= 0.0 && response.quality_score <= 1.0,
            "Failed with {}",
            description
        );
    }
}

#[test]
fn test_integration_overflow_number() {
    let input = "4294967296"; // u32::MAX + 1
    let response = validate_random_numbers(input);
    assert!(!response.valid);
    assert_eq!(response.quality_score, 0.0);
}

#[test]
fn test_integration_without_nist() {
    let input = "1,2,3,4,5,6,7,8,9,10";
    let response = validate_random_numbers_with_nist(input, false);

    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
    assert!(response.nist_results.is_some());
    assert!(response
        .nist_results
        .unwrap()
        .contains("not requested"));
}

#[test]
fn test_integration_validation_request_structure() {
    // Test that ValidationRequest can be created and serialized
    let request = ValidationRequest {
        numbers: "1,2,3".to_string(),
        use_nist: true,
    };

    assert_eq!(request.numbers, "1,2,3");
    assert!(request.use_nist);

    // Test default use_nist
    let json = r#"{"numbers":"1,2,3"}"#;
    let parsed: ValidationRequest = serde_json::from_str(json).unwrap();
    assert!(parsed.use_nist); // Should default to true
}

#[test]
fn test_integration_response_serialization() {
    let input = "1,2,3,4,5";
    let response = validate_random_numbers(input);

    // Test that response can be serialized to JSON
    let json = serde_json::to_string(&response);
    assert!(json.is_ok());

    // Test that it can be deserialized back
    let json_str = json.unwrap();
    let parsed: Result<randomnumbervalidator::ValidationResponse, _> =
        serde_json::from_str(&json_str);
    assert!(parsed.is_ok());
}

#[test]
fn test_integration_very_large_input() {
    // Test with 500 numbers
    let numbers: Vec<String> = (1..=500).map(|n| n.to_string()).collect();
    let input = numbers.join(",");
    let response = validate_random_numbers(&input);

    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
    assert!(response.message.contains("bits"));
}

#[test]
fn test_integration_repeating_pattern() {
    // Test with obvious pattern (should get low quality score)
    let input = "1,1,1,1,1,1,1,1,1,1";
    let response = validate_random_numbers(input);

    // Repeating pattern should have lower quality
    assert!(response.quality_score < 0.7);
}

#[test]
fn test_integration_alternating_pattern() {
    // Test with alternating pattern
    let input = "0,4294967295,0,4294967295,0,4294967295"; // alternating all 0s and all 1s
    let response = validate_random_numbers(input);

    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
}

#[test]
fn test_integration_whitespace_handling() {
    let inputs = vec![
        "  1  ,  2  ,  3  ",     // spaces around numbers
        "\n1\n,\n2\n,\n3\n",     // newlines everywhere
        "\t1,\t2,\t3",           // tabs before numbers
        "  \n\t1 , 2\n,\t 3  \n", // mixed whitespace
    ];

    for input in inputs {
        let response = validate_random_numbers(input);
        assert!(
            response.quality_score >= 0.0 && response.quality_score <= 1.0,
            "Failed to handle whitespace in: {:?}",
            input
        );
    }
}

#[test]
fn test_integration_single_number() {
    let response = validate_random_numbers("42");
    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
}

#[test]
fn test_integration_duplicate_numbers() {
    // Duplicate numbers should still be processed
    let input = "5,5,5,10,10,10,15,15,15";
    let response = validate_random_numbers(input);
    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
}
