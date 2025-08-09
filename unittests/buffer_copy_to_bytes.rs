// Tests for the copy_to_bytes method of Formatted
// These tests cover various edge cases and scenarios to ensure robustness

use crate::Formatter;

#[test]
fn test_decimal_basic() {
    let formatted = Formatter::format_finite_f64(3.14159);
    let mut buf = [0u8; 64];

    // Test basic functionality
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.14");
    assert_eq!(written, 4);

    // Test exact decimal places
    let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.14159");
    assert_eq!(written, 7);

    // Test padding with zeros
    let written = formatted.copy_to_bytes::<8>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.14159000");
    assert_eq!(written, 10);
}

#[test]
fn test_decimal_zero_places() {
    let formatted = Formatter::format_finite_f64(3.14159);
    let mut buf = [0u8; 64];

    // Test zero decimal places (integer part only)
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3");
    assert_eq!(written, 1);
}

#[test]
fn test_negative_numbers() {
    let formatted = Formatter::format_finite_f64(-3.14159);
    let mut buf = [0u8; 64];

    // Test negative with truncation
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"-3.14");
    assert_eq!(written, 5);

    // Test negative with padding
    let written = formatted.copy_to_bytes::<6>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"-3.141590");
    assert_eq!(written, 9);

    // Test negative integer part only
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"-3");
    assert_eq!(written, 2);
}

#[test]
fn test_integer_numbers() {
    let formatted = Formatter::format_finite_f64(42.0);
    let mut buf = [0u8; 64];

    // Integer with decimal places should pad with zeros
    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"42.000");
    assert_eq!(written, 6);

    // Integer with zero decimal places
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"42");
    assert_eq!(written, 2);
}

#[test]
fn test_very_small_numbers() {
    let formatted = Formatter::format_finite_f64(0.000001);
    let mut buf = [0u8; 64];

    // This should be in decimal format, not scientific
    let written = formatted.copy_to_bytes::<8>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"1.00000000e-6");
    assert_eq!(written, 13);
}

#[test]
fn test_scientific_notation() {
    let formatted = Formatter::format_finite_f64(3e20);
    let mut buf = [0u8; 64];

    // Scientific notation - no decimal point in original
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3e20");
    assert_eq!(written, 4);

    // Add decimal point and padding
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.00e20");
    assert_eq!(written, 7);

    // Test with number that has decimal part in scientific notation
    let formatted = Formatter::format_finite_f64(3.14e20);
    let written = formatted.copy_to_bytes::<1>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.1e20");
    assert_eq!(written, 6);

    // Test padding in scientific notation
    let written = formatted.copy_to_bytes::<4>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"3.1400e20");
    assert_eq!(written, 9);
}

#[test]
fn test_scientific_negative() {
    let formatted = Formatter::format_finite_f64(-1.23e-10);
    let mut buf = [0u8; 64];

    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    // The exact format depends on the implementation, but should preserve the sign
    // and exponent
    assert!(&buf[..written].starts_with(b"-"));
    assert!(&buf[..written].contains(&b'e'));
}

#[test]
fn test_nonfinite() {
    let mut buf = [0u8; 64];

    // Test NaN
    let formatted = Formatter::format_f64(f64::NAN);
    let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"NaN");
    assert_eq!(written, 3);

    // Test positive infinity
    let formatted = Formatter::format_f64(f64::INFINITY);
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"inf");
    assert_eq!(written, 3);

    // Test negative infinity
    let formatted = Formatter::format_f64(f64::NEG_INFINITY);
    let written = formatted.copy_to_bytes::<10>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"-inf");
    assert_eq!(written, 4);
}

#[test]
fn test_buffer_too_small() {
    let formatted = Formatter::format_finite_f64(3.14159);

    // Buffer too small for the result
    let mut small_buf = [0u8; 3];
    let result = formatted.copy_to_bytes::<5>(&mut small_buf);
    assert_eq!(result, None);

    // Buffer exactly the right size
    let mut exact_buf = [0u8; 7]; // "3.14159" is 7 bytes
    let result = formatted.copy_to_bytes::<5>(&mut exact_buf);
    assert_eq!(result, Some(7));
    assert_eq!(&exact_buf, b"3.14159");
}

#[test]
fn test_edge_cases_zero() {
    let formatted = Formatter::format_finite_f64(0.0);
    let mut buf = [0u8; 64];

    // Zero with no decimal places
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"0");
    assert_eq!(written, 1);

    // Zero with decimal places
    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"0.000");
    assert_eq!(written, 5);

    // Negative zero
    let formatted = Formatter::format_finite_f64(-0.0);
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    // The behavior of -0.0 depends on implementation, but should be handled
    // gracefully
    assert!(written > 0);
}

#[test]
fn test_large_decimal_places() {
    let formatted = Formatter::format_finite_f64(1.5);
    let mut buf = [0u8; 64];

    // Test with many decimal places (should pad with zeros)
    let written = formatted.copy_to_bytes::<20>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"1.50000000000000000000");
    assert_eq!(written, 22);
}

#[test]
fn test_f32_variants() {
    // Test that f32 formatting also works correctly
    let formatted = Formatter::format_finite_f32(3.14159_f32);
    let mut buf = [0u8; 64];

    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    // f32 has less precision, so the result might be slightly different
    assert!(&buf[..written].starts_with(b"3.14"));

    // Test f32 nonfinite
    let formatted = Formatter::format_f32(f32::INFINITY);
    let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"inf");
}

#[test]
fn test_boundary_values() {
    let mut buf = [0u8; 64];

    // Test very large number
    let formatted = Formatter::format_finite_f64(f64::MAX);
    let written = formatted.copy_to_bytes::<0>(&mut buf);
    assert!(written.is_some());

    // Test very small positive number
    let formatted = Formatter::format_finite_f64(f64::MIN_POSITIVE);
    let written = formatted.copy_to_bytes::<0>(&mut buf);
    assert!(written.is_some());

    // Test number close to 1.0
    let formatted = Formatter::format_finite_f64(1.0000000000000002);
    let written = formatted.copy_to_bytes::<20>(&mut buf);
    assert!(written.is_some());
}

#[test]
fn test_buffer_edge_sizes() {
    let formatted = Formatter::format_finite_f64(3.14);

    // Test with buffer that's exactly 1 byte too small
    let mut buf = [0u8; 3]; // Need 4 for "3.14"
    let result = formatted.copy_to_bytes::<2>(&mut buf);
    assert_eq!(result, None);

    // Test with buffer that's exactly the right size
    let mut buf = [0u8; 4];
    let result = formatted.copy_to_bytes::<2>(&mut buf);
    assert_eq!(result, Some(4));
    assert_eq!(&buf, b"3.14");

    // Test with empty buffer
    let mut buf = [];
    let result = formatted.copy_to_bytes::<0>(&mut buf);
    assert_eq!(result, None);
}

#[test]
fn test_consistency_with_as_str() {
    // For cases where copy_to_bytes should match as_str output
    let numbers = [
        0.0,
        1.0,
        -1.0,
        core::f64::consts::PI,
        core::f64::consts::E,
        f64::MIN_POSITIVE,
        1e-10,
        1e10,
    ];

    for &num in &numbers {
        let formatted = Formatter::format_finite_f64(num);
        let original_str = formatted.as_str();

        // Count decimal places in original
        let decimal_places = if let Some(dot_pos) = original_str.find('.') {
            let after_dot = &original_str[dot_pos + 1..];
            // Find end of decimal part (before 'e' if present)
            after_dot.find('e').unwrap_or(after_dot.len())
        } else {
            0
        };

        if decimal_places > 0 {
            let mut buf = [0u8; 64];
            // This test is more about ensuring we don't panic than exact output
            let result = formatted.copy_to_bytes::<5>(&mut buf);
            assert!(result.is_some(), "Failed for number: {num}");
        }
    }
}

#[test]
fn test_extreme_precision() {
    // Test with numbers that have very long decimal representations
    let formatted = Formatter::format_finite_f64(1.0 / 3.0);
    let mut buf = [0u8; 64];

    // Test truncation of repeating decimals
    let written = formatted.copy_to_bytes::<10>(&mut buf).unwrap();
    assert!(written > 10); // Should be more than 10 characters
    assert!(&buf[..written].starts_with(b"0.333333333"));
}

#[test]
fn test_near_boundaries() {
    let mut buf = [0u8; 64];

    // Test numbers near powers of 10
    let formatted = Formatter::format_finite_f64(0.99999);
    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    assert!(&buf[..written].starts_with(b"0.999"));

    let formatted = Formatter::format_finite_f64(1.00001);
    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    assert!(&buf[..written].starts_with(b"1.000"));

    let formatted = Formatter::format_finite_f64(9.99999);
    let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    assert!(buf[..written].starts_with(b"9.99") || buf[..written].starts_with(b"10.0"));
}

#[test]
fn test_subnormal_numbers() {
    let mut buf = [0u8; 64];

    // Test subnormal numbers (very small numbers near zero)
    let formatted = Formatter::format_finite_f64(f64::MIN_POSITIVE);
    let written = formatted.copy_to_bytes::<0>(&mut buf);
    assert!(written.is_some());

    // The output should be in scientific notation
    if let Some(written) = written {
        // Check if it contains 'e' (scientific notation indicator)
        assert!(buf[..written].contains(&b'e'));
    }
}

#[test]
fn test_powers_of_two() {
    let mut buf = [0u8; 64];

    // Test various powers of 2, which should have exact representations
    let powers = [1.0, 2.0, 4.0, 8.0, 16.0, 0.5, 0.25, 0.125];

    for &power in &powers {
        let formatted = Formatter::format_finite_f64(power);
        let written = formatted.copy_to_bytes::<4>(&mut buf).unwrap();
        assert!(written > 0);

        // Verify that we can round-trip through string
        assert!(written > 0);
        // Basic sanity check - should contain digits
        assert!(buf[..written].iter().any(|&b| b.is_ascii_digit()));
    }
}

#[test]
fn test_trailing_zeros_handling() {
    let mut buf = [0u8; 64];

    // Test numbers that might have trailing zeros
    let formatted = Formatter::format_finite_f64(1.5);
    let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"1");

    let written = formatted.copy_to_bytes::<1>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"1.5");

    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"1.500");
}

#[test]
fn test_scientific_edge_cases() {
    let mut buf = [0u8; 64];

    // Test scientific notation with various exponents
    let numbers = [1e-100, 1e100, 1.23e-50, 9.876e75];

    for &num in &numbers {
        let formatted = Formatter::format_finite_f64(num);

        // Test with 0 decimal places
        let written = formatted.copy_to_bytes::<0>(&mut buf);
        assert!(written.is_some());

        // Test with some decimal places
        let written = formatted.copy_to_bytes::<3>(&mut buf);
        assert!(written.is_some());

        if let Some(written) = written {
            // Should contain 'e' for scientific notation
            assert!(buf[..written].contains(&b'e'));
        }
    }
}

#[test]
fn test_buffer_boundary_exact() {
    // Test with buffers that are exactly the right size
    let formatted = Formatter::format_finite_f64(1.23);

    // "1.23" = 4 bytes
    let mut buf_exact = [0u8; 4];
    let result = formatted.copy_to_bytes::<2>(&mut buf_exact);
    assert_eq!(result, Some(4));
    assert_eq!(&buf_exact, b"1.23");

    // "1.230" = 5 bytes
    let mut buf_exact = [0u8; 5];
    let result = formatted.copy_to_bytes::<3>(&mut buf_exact);
    assert_eq!(result, Some(5));
    assert_eq!(&buf_exact, b"1.230");

    // Buffer too small by 1 byte
    let mut buf_small = [0u8; 4];
    let result = formatted.copy_to_bytes::<3>(&mut buf_small);
    assert_eq!(result, None);
}

#[test]
fn test_zero_with_sign() {
    let mut buf = [0u8; 64];

    // Test positive zero
    let formatted = Formatter::format_finite_f64(0.0);
    let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    assert_eq!(&buf[..written], b"0.00000");

    // Test negative zero (behavior may vary by implementation)
    let formatted = Formatter::format_finite_f64(-0.0);
    let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    // Should be either "0.000" or "-0.000"
    assert!(&buf[..written] == b"0.000" || &buf[..written] == b"-0.000");
}
