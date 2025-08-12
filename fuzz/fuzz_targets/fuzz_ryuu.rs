#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
#[derive(Arbitrary)]
enum FloatInput {
    F32(f32),
    F64(f64),
}

macro_rules! ryuu_test {
    ($val:expr, $method:ident, $ty:ty) => {
        match $val {
            val => {
                let formatted = ryuu::Formatter::$method(val);
                let mut formatted_2 = ryuu::Formatter::$method(val);

                // Basic
                let string = formatted.as_str();

                // Fixed decimal places
                let string_fixed_dp = formatted.as_str_fixed_dp::<2>();

                // Fixed decimal places
                let string_fixed_dp_2 = formatted_2.as_str_adjusting_dp::<2>();

                // Fixed decimal places
                let string_fixed_dp_copy_to_bytes = {
                    let mut string_fixed_dp_buf = String::new();

                    {
                        let string_fixed_dp_buf = unsafe {
                            let vec = string_fixed_dp_buf.as_mut_vec();
                            vec.reserve(32 + 2);
                            vec.set_len(32 + 2);
                            vec
                        };

                        let actual_size = formatted
                            .copy_to_bytes::<2>(&mut string_fixed_dp_buf[..])
                            .expect("Buffer too small");

                        unsafe {
                            string_fixed_dp_buf.set_len(actual_size);
                        }
                    }

                    string_fixed_dp_buf
                };

                if val.is_finite() {
                    // General, as-is
                    let string_parsed = string
                        .parse::<$ty>()
                        .unwrap_or_else(|_| panic!("Failed to parse: {}", string_fixed_dp_copy_to_bytes));

                    assert_eq!(val, string_parsed);

                    // Fixed decimal places, but ignore exponent / inf
                    let string_fixed_dp_parsed = string_fixed_dp
                        .parse::<$ty>()
                        .unwrap_or_else(|_| panic!("Failed to parse: {}", string_fixed_dp));

                    assert!(
                        (val - string_fixed_dp_parsed).abs() <= 0.010_000_000_000_001,
                        "val: {}, string_fixed_dp_parsed: {}",
                        val,
                        string_fixed_dp_parsed
                    );

                    // Fixed decimal places, but adjusted to 2 decimal places (excluding exponent /
                    // inf)
                    let string_fixed_dp_2_parsed = string_fixed_dp_2
                        .parse::<$ty>()
                        .unwrap_or_else(|_| panic!("Failed to parse: {}", string_fixed_dp_2));

                    if !string_fixed_dp_2.contains('e') {
                        assert!(
                            string_fixed_dp_2.rsplit_once('.').is_some(),
                            "string_fixed_dp_2 should contain 2 decimal places: {}",
                            string_fixed_dp_2
                        );
                    }

                    assert!(
                        (val - string_fixed_dp_2_parsed).abs() <= 0.010_000_000_000_001,
                        "val: {}, string_fixed_dp_2_parsed: {}",
                        val,
                        string_fixed_dp_2_parsed
                    );

                    // Fixed decimal places, but adjusted to 2 decimal places (including exponent)
                    let string_fixed_dp_copy_to_bytes_parsed = string_fixed_dp_copy_to_bytes
                        .parse::<$ty>()
                        .unwrap_or_else(|_| panic!("Failed to parse: {}", string_fixed_dp_copy_to_bytes));

                    // Control value for comparison, handling exponent
                    let control = if let Some((coeff, exp)) = string_fixed_dp_copy_to_bytes.split_once("e") {
                        let _coeff = coeff
                            .parse::<$ty>()
                            .unwrap_or_else(|_| panic!("Failed to parse coefficient: {}", coeff));
                        let exp = exp
                            .parse::<i32>()
                            .unwrap_or_else(|_| panic!("Failed to parse exponent: {}", exp));

                        (0.010_000_000_000_001 * 10.0f64.powi(exp)).max(f64::MIN_POSITIVE)
                    } else {
                        0.010_000_000_000_001
                    };

                    assert!(
                        (val - string_fixed_dp_copy_to_bytes_parsed).abs() as f64 <= control,
                        "val: {}, string_fixed_dp_copy_to_bytes_parsed: {}, control: {}",
                        val,
                        string_fixed_dp_copy_to_bytes_parsed,
                        control
                    );
                }
            }
        }
    };
}

fuzz_target!(|inputs: (FloatInput, bool)| {
    let (input, finite) = inputs;
    match (input, finite) {
        (FloatInput::F32(val), false) => ryuu_test!(val, format_f32, f32),
        (FloatInput::F32(val), true) => ryuu_test!(val, format_finite_f32, f32),
        (FloatInput::F64(val), false) => ryuu_test!(val, format_f64, f64),
        (FloatInput::F64(val), true) => ryuu_test!(val, format_finite_f64, f64),
    }
});
