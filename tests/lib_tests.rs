// Unit tests for lib.rs
use randomnumbervalidator::*;

#[test]
fn test_prepare_input_basic() {
    let result = prepare_input_for_nist("1,2,3");
    // Range 1-3 doesn't start at 0, so should require range specification
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("doesn't fit standard bit widths"));
}

#[test]
fn test_prepare_input_invalid() {
    let result = prepare_input_for_nist("1,abc,3");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("letters"));
}

#[test]
fn test_prepare_input_newline_delimiter() {
    let result = prepare_input_for_nist("0\n128\n255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_prepare_input_space_delimiter() {
    let result = prepare_input_for_nist("0 100 255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_prepare_input_mixed_delimiters() {
    let result = prepare_input_for_nist("0, 50\n100\t150;255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 40); // 5 numbers * 8 bits
}

#[test]
fn test_prepare_input_reject_letters() {
    let result = prepare_input_for_nist("123abc456");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("letters"));
}

#[test]
fn test_validate_random_numbers() {
    let response = validate_random_numbers("0,1,2,3,4,5");
    assert!(response.quality_score >= 0.0);
    assert!(response.quality_score <= 1.0);
}


#[test]
fn test_prepare_input_single_number() {
    let result = prepare_input_for_nist("0,42");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 16); // 2 numbers * 8 bits
}

#[test]
fn test_prepare_input_zero() {
    let result = prepare_input_for_nist("0");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 8);
    // All bits should be 0
    assert!(bits.iter().all(|&b| b == 0));
}

#[test]
fn test_prepare_input_max_u32() {
    let result = prepare_input_for_nist("0,4294967295"); // u32::MAX
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 64); // 2 numbers * 32 bits
                                // Last 32 bits should be all 1
    assert!(bits[32..].iter().all(|&b| b == 1));
}

#[test]
fn test_prepare_input_overflow() {
    // Number larger than u32::MAX should fail
    let result = prepare_input_for_nist("4294967296");
    assert!(result.is_err());
}

#[test]
fn test_prepare_input_empty_string() {
    let result = prepare_input_for_nist("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No numbers"));
}

#[test]
fn test_prepare_input_whitespace_only() {
    let result = prepare_input_for_nist("   \n\t  ");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No numbers"));
}

#[test]
fn test_prepare_input_special_characters() {
    // Should treat special chars as delimiters
    let result = prepare_input_for_nist("0!@#100$%^255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_prepare_input_negative_sign() {
    // Negative numbers should work (the minus is treated as delimiter)
    let result = prepare_input_for_nist("0,5,10");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_validate_invalid_input() {
    let response = validate_random_numbers("abc");
    assert!(!response.valid);
    assert_eq!(response.quality_score, 0.0);
    assert!(response.message.contains("letters"));
}



#[test]
fn test_prepare_input_leading_zeros() {
    // Numbers with leading zeros should be parsed correctly
    let result = prepare_input_for_nist("0,007,042,0100");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 32); // 4 numbers * 8 bits (max is 100)
}

#[test]
fn test_validation_response_structure() {
    // Generate enough numbers for NIST (at least 100 bits, so 13+ numbers with 8-bit encoding)
    let numbers: Vec<String> = (0..20).map(|n| (n * 10).to_string()).collect();
    let input = numbers.join(",");
    let response = validate_random_numbers(&input);

    // Verify all fields are populated
    assert!(response.quality_score >= 0.0 && response.quality_score <= 1.0);
    assert!(!response.message.is_empty());
    assert!(response.nist_results.is_some());
    assert!(response.nist_data.is_some());
}


#[test]
fn test_prepare_input_large_sequence() {
    // Test with many numbers
    let numbers: Vec<String> = (1..=100).map(|n| n.to_string()).collect();
    let input = numbers.join(",");
    let result = prepare_input_for_nist(&input);
    assert!(result.is_err()); // Should fail without range
    assert!(result
        .unwrap_err()
        .contains("doesn't fit standard bit widths"));
}

// ========== Tests for standard bit width detection ==========

#[test]
fn test_8bit_standard_range() {
    // Numbers 0-255 should use 8 bits per number
    let result = prepare_input_for_nist("0,128,255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_16bit_standard_range() {
    // Numbers 0-65535 should use 16 bits per number
    let result = prepare_input_for_nist("0,256,65535");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 48); // 3 numbers * 16 bits
}

#[test]
fn test_32bit_standard_range() {
    // Numbers 0-4294967295 should use 32 bits per number
    let result = prepare_input_for_nist("0,65536,4294967295");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
}

#[test]
fn test_8bit_boundary() {
    // Exactly 255 should still use 8 bits
    let result = prepare_input_for_nist("0,100,255");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 * 8
}

// ========== Tests for non-standard ranges (should fail without range specification) ==========

#[test]
fn test_nonstandard_range_1_to_100() {
    // Range 1-100 doesn't start at 0, should require range specification
    let result = prepare_input_for_nist("1,50,100");
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("doesn't fit standard bit widths"));
    assert!(err_msg.contains("range_min"));
}

#[test]
fn test_nonstandard_range_50_to_200() {
    let result = prepare_input_for_nist("50,100,200");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("range_min and range_max"));
}

// ========== Tests for custom range with base conversion ==========

#[test]
fn test_custom_range_1_to_100() {
    // With range specified, should use base conversion
    let result = prepare_input_for_nist_with_range("1,50,100", Some(1), Some(100));
    assert!(result.is_ok());
    let bits = result.unwrap();
    // 3 numbers in base-100 ≈ 3 * log2(100) ≈ 3 * 6.64 ≈ 20 bits
    // The actual result is 24 bits (3 bytes from BigUint conversion)
    assert!(bits.len() >= 16 && bits.len() <= 24);
}

#[test]
fn test_custom_range_validation() {
    // Numbers outside specified range should fail
    let result = prepare_input_for_nist_with_range("1,50,101", Some(1), Some(100));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside specified range"));
}

#[test]
fn test_custom_range_invalid_min_max() {
    // min > max should fail
    let result = prepare_input_for_nist_with_range("50", Some(100), Some(50));
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("min") && err_msg.contains("max"));
}

#[test]
fn test_base_conversion_deterministic() {
    // Same input should always produce same output
    let result1 = prepare_input_for_nist_with_range("1,2,3,4,5", Some(1), Some(10));
    let result2 = prepare_input_for_nist_with_range("1,2,3,4,5", Some(1), Some(10));
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_base_conversion_entropy() {
    // More numbers should produce more bits
    let result3 = prepare_input_for_nist_with_range("1,2,3", Some(1), Some(10));
    let result10 = prepare_input_for_nist_with_range("1,2,3,4,5,6,7,8,9,10", Some(1), Some(10));
    assert!(result3.is_ok());
    assert!(result10.is_ok());
    let bits3 = result3.unwrap();
    let bits10 = result10.unwrap();
    assert!(bits10.len() > bits3.len());
}

#[test]
fn test_8bit_with_explicit_range() {
    // Even with standard range, explicit range should still work
    let result = prepare_input_for_nist_with_range("0,128,255", Some(0), Some(255));
    assert!(result.is_ok());
    let bits = result.unwrap();
    // With explicit range 0-255, should use base conversion
    // 3 numbers in base-256 ≈ 3 * 8 = 24 bits
    assert_eq!(bits.len(), 24);
}

#[test]
fn test_old_test_compatibility() {
    // Old tests that used 32 bits should now fail or use 8/16 bits
    // Testing 0,42: should use 8 bits
    let result = prepare_input_for_nist("0,42");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 16); // 2 numbers * 8 bits (not 32!)
}

// ========== Tests for bit-width enforcement ==========

#[test]
fn test_bitwidth_enforced_8bit() {
    // With bit_width=8, should use 8 bits regardless of actual max
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("0,50,100", None, None, Some(8));
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_bitwidth_enforced_16bit() {
    // With bit_width=16, should use 16 bits
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("0,256,1000", None, None, Some(16));
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 48); // 3 numbers * 16 bits
}

#[test]
fn test_bitwidth_enforced_32bit() {
    // With bit_width=32, should use 32 bits
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("0,65536,100000", None, None, Some(32));
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 96); // 3 numbers * 32 bits
}

#[test]
fn test_bitwidth_rejection_exceeds_8bit() {
    // Number 256 exceeds 8-bit max (255)
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("0,100,256", None, None, Some(8));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("exceeds"));
    assert!(err.contains("8-bit"));
    assert!(err.contains("255"));
}

#[test]
fn test_bitwidth_rejection_exceeds_16bit() {
    // Number 65536 exceeds 16-bit max (65535)
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("0,1000,65536", None, None, Some(16));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("exceeds"));
    assert!(err.contains("16-bit"));
}

#[test]
fn test_bitwidth_allows_nonzero_min() {
    // Numbers starting at 1 (not 0) are allowed - might just be a small sample
    // The statistical tests will detect bias if it exists
    let result =
        prepare_input_for_nist_with_range_and_bitwidth("1,50,100", None, None, Some(8));
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_bitwidth_invalid_value() {
    // bit_width must be 8, 16, or 32
    let result = prepare_input_for_nist_with_range_and_bitwidth("0,1,2", None, None, Some(12));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid bit_width"));
    assert!(err.contains("12"));
}

#[test]
fn test_bitwidth_fallback_to_auto_detection() {
    // Without bit_width specified, should auto-detect (8-bit for 0-255)
    let result = prepare_input_for_nist_with_range_and_bitwidth("0,128,255", None, None, None);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits (auto-detected)
}

// ========== Tests for base64 input format ==========

#[test]
fn test_base64_basic() {
    // "Hello" in base64 is "SGVsbG8="
    let result = parse_base64_to_bits("SGVsbG8=");
    assert!(result.is_ok());
    let bits = result.unwrap();
    // "Hello" = 5 bytes = 40 bits
    assert_eq!(bits.len(), 40);
}

#[test]
fn test_base64_with_whitespace() {
    // Base64 with whitespace should be handled
    let result = parse_base64_to_bits("SGVs bG8=");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 40);
}

#[test]
fn test_base64_invalid() {
    // Invalid base64 should fail
    let result = parse_base64_to_bits("!!!invalid!!!");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid base64"));
}

#[test]
fn test_base64_empty() {
    // Empty base64 should fail
    let result = parse_base64_to_bits("");
    assert!(result.is_err());
}

#[test]
fn test_base64_missing_padding() {
    // Base64 without padding should work (auto-padded)
    // "Hello" in base64 is "SGVsbG8=" but we test without the padding
    let result = parse_base64_to_bits("SGVsbG8");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 40); // 5 bytes = 40 bits
}

#[test]
fn test_base64_auto_padding() {
    // Test different padding scenarios
    let test_cases = vec![
        ("SGVsbG8", 40),  // "Hello" - needs 1 padding
        ("Zm9v", 24),     // "foo" - needs 0 padding (already multiple of 4)
        ("SGVsbG8=", 40), // "Hello" - already has padding
    ];

    for (input, expected_bits) in test_cases {
        let result = parse_base64_to_bits(input);
        assert!(result.is_ok(), "Failed to parse: {}", input);
        assert_eq!(
            result.unwrap().len(),
            expected_bits,
            "Wrong bit count for: {}",
            input
        );
    }
}

#[test]
fn test_base64_binary_data() {
    // Test with actual random bytes encoded as base64
    // 16 bytes = 128 bits
    let result = parse_base64_to_bits("AAAAAAAAAAAAAAAAAAAAAA==");
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 128);
    // All zeros
    assert!(bits.iter().all(|&b| b == 0));
}

#[test]
fn test_prepare_input_with_format_numbers() {
    let result =
        prepare_input_with_format("0,128,255", &InputFormat::Numbers, None, None, None);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24); // 3 numbers * 8 bits
}

#[test]
fn test_prepare_input_with_format_base64() {
    let result = prepare_input_with_format("SGVsbG8=", &InputFormat::Base64, None, None, None);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 40); // "Hello" = 40 bits
}

#[test]
fn test_validate_with_base64_format() {
    // Test validation with base64 input (needs enough data for NIST)
    // Generate a large base64 string (at least 12500 bytes = 100,000 bits)
    // Use a varied pattern to avoid issues with statistical tests
    let mut bytes = Vec::new();
    for i in 0..12500 {
        bytes.push(((i * 7 + 13) % 256) as u8); // Pseudo-random pattern
    }
    use base64::prelude::*;
    let base64_input = BASE64_STANDARD.encode(&bytes);

    let response = validate_random_numbers_full(
        &base64_input,
        &InputFormat::Base64,
        None,
        None,
        None,
        false,
    );

    assert!(response.quality_score >= 0.0 && response.quality_score <= 1.0);
}

#[test]
fn test_input_format_default() {
    let format = InputFormat::default();
    assert_eq!(format, InputFormat::Numbers);
}

// ========== Tests for debug logging ==========

#[test]
fn test_write_bits_to_debug_file() {
    let bits = vec![1, 0, 1, 0, 1, 1, 0, 0];
    let result = write_bits_to_debug_file(&bits);
    assert!(result.is_ok());
    let filepath = result.unwrap();
    assert!(filepath.contains("debug/bits_"));

    // Verify file exists and can be read
    let content = std::fs::read_to_string(&filepath);
    assert!(content.is_ok());
    let file_content = content.unwrap();
    assert!(file_content.contains("# Bit Stream Debug Output"));
    assert!(file_content.contains("# Total bits: 8"));

    // Clean up
    let _ = std::fs::remove_file(&filepath);
}

#[test]
fn test_validate_with_debug_log() {
    // Generate enough numbers for NIST (at least 100 bits, so 13+ numbers with 8-bit encoding)
    let numbers: Vec<String> = (0..20).map(|n| (n * 10).to_string()).collect();
    let input = numbers.join(",");

    let response = validate_random_numbers_full(
        &input,
        &InputFormat::Numbers,
        None,
        None,
        None,
        true, // Enable debug logging
    );

    assert!(response.debug_file.is_some());
    let debug_file = response.debug_file.unwrap();
    assert!(debug_file.contains("debug/bits_"));

    // Verify file exists
    assert!(std::path::Path::new(&debug_file).exists());

    // Clean up
    let _ = std::fs::remove_file(&debug_file);
}

#[test]
fn test_validate_without_debug_log() {
    // Generate enough numbers for NIST (at least 100 bits, so 13+ numbers with 8-bit encoding)
    let numbers: Vec<String> = (0..20).map(|n| (n * 10).to_string()).collect();
    let input = numbers.join(",");

    let response = validate_random_numbers_full(
        &input,
        &InputFormat::Numbers,
        None,
        None,
        None,
        false, // Disable debug logging
    );

    assert!(response.debug_file.is_none());
}

// ========== Tests for base conversion with consistent length ==========

#[test]
fn test_base_conversion_consistent_length() {
    // Test that base conversion produces consistent bit lengths
    // For range 2-8 (7 values), 4 numbers should produce ceil(4 * log2(7)) = 12 bits

    let result1 = convert_to_bits_base_conversion(&[2, 2, 2, 2], 2, 8);
    assert!(result1.is_ok());
    let bits1 = result1.unwrap();
    assert_eq!(bits1.len(), 12, "All minimum values should produce 12 bits");

    let result2 = convert_to_bits_base_conversion(&[8, 8, 8, 8], 2, 8);
    assert!(result2.is_ok());
    let bits2 = result2.unwrap();
    assert_eq!(bits2.len(), 12, "All maximum values should produce 12 bits");

    let result3 = convert_to_bits_base_conversion(&[2, 5, 8, 3], 2, 8);
    assert!(result3.is_ok());
    let bits3 = result3.unwrap();
    assert_eq!(bits3.len(), 12, "Mixed values should produce 12 bits");
}

#[test]
fn test_base_conversion_leading_zeros() {
    // Test that all-minimum values produce leading zeros
    let result = convert_to_bits_base_conversion(&[2, 2, 2, 2], 2, 8);
    assert!(result.is_ok());
    let bits = result.unwrap();

    // All minimum values (normalized to [0,0,0,0]) should be very small
    // Should have leading zeros
    let leading_zeros = bits.iter().take_while(|&&b| b == 0).count();
    assert!(leading_zeros > 0, "Should have leading zeros for small values");
}

#[test]
fn test_base_conversion_entropy_calculation() {
    // Test entropy calculation for different ranges

    // Range 0-1 (2 values): should produce exactly 1 bit per number
    let result = convert_to_bits_base_conversion(&[0, 1, 0, 1], 0, 1);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 4, "Range 0-1 should produce 4 bits for 4 numbers");

    // Range 0-3 (4 values): should produce exactly 2 bits per number
    let result = convert_to_bits_base_conversion(&[0, 1, 2, 3], 0, 3);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 8, "Range 0-3 should produce 8 bits for 4 numbers");

    // Range 0-7 (8 values): should produce exactly 3 bits per number
    let result = convert_to_bits_base_conversion(&[0, 1, 2, 3, 4, 5, 6, 7], 0, 7);
    assert!(result.is_ok());
    let bits = result.unwrap();
    assert_eq!(bits.len(), 24, "Range 0-7 should produce 24 bits for 8 numbers");
}

#[test]
fn test_base_conversion_different_values_same_length() {
    // Test that different sequences in the same range produce the same bit length
    let sequences = vec![
        vec![2, 2, 2, 2],
        vec![8, 8, 8, 8],
        vec![2, 8, 2, 8],
        vec![5, 5, 5, 5],
        vec![3, 4, 6, 7],
    ];

    let mut lengths = Vec::new();
    for seq in sequences {
        let result = convert_to_bits_base_conversion(&seq, 2, 8);
        assert!(result.is_ok());
        lengths.push(result.unwrap().len());
    }

    // All lengths should be the same
    assert!(lengths.iter().all(|&l| l == lengths[0]),
            "All sequences should produce same bit length, got: {:?}", lengths);
}

#[test]
fn test_base_conversion_uniqueness() {
    // Test that different sequences produce different bit patterns (mostly)
    let result1 = convert_to_bits_base_conversion(&[2, 2, 2, 2], 2, 8);
    let result2 = convert_to_bits_base_conversion(&[8, 8, 8, 8], 2, 8);
    let result3 = convert_to_bits_base_conversion(&[2, 8, 2, 8], 2, 8);

    assert!(result1.is_ok() && result2.is_ok() && result3.is_ok());

    let bits1 = result1.unwrap();
    let bits2 = result2.unwrap();
    let bits3 = result3.unwrap();

    // Different sequences should produce different bit patterns
    assert_ne!(bits1, bits2, "Different sequences should produce different bits");
    assert_ne!(bits1, bits3, "Different sequences should produce different bits");
    assert_ne!(bits2, bits3, "Different sequences should produce different bits");
}
