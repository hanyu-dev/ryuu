#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
enum FloatInput {
    F32(f32),
    F64(f64),
}

macro_rules! ryuu_test {
    ($val:expr, $method:ident) => {
        match $val {
            val => {
                let formatted = ryuu::Formatter::$method(val);
                let string = formatted.as_str();
                if val.is_finite() {
                    assert_eq!(val, string.parse().unwrap());
                }
            }
        }
    };
}

fuzz_target!(|inputs: (FloatInput, bool)| {
    let (input, finite) = inputs;
    match (input, finite) {
        (FloatInput::F32(val), false) => ryuu_test!(val, format_f32),
        (FloatInput::F32(val), true) => ryuu_test!(val, format_finite_f32),
        (FloatInput::F64(val), false) => ryuu_test!(val, format_f64),
        (FloatInput::F64(val), true) => ryuu_test!(val, format_finite_f64),
    }
});
