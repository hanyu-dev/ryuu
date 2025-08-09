//! Safe API for formatting floating point numbers to text.

use core::mem::MaybeUninit;
use core::{fmt, ops, ptr, slice, str};

use crate::raw::{self, FormattedMeta};

/// The length of the buffer used to store the formatted text.
pub const BUFFER_LEN: usize = 32;

#[derive(Debug, Clone, Copy)]
/// Safe API for formatting floating point numbers to text.
///
/// ## Example
///
/// ```
/// assert_eq!(
///     ryuu::Formatter::format_finite_f64(1.234_f64).as_str(),
///     "1.234"
/// );
/// assert_eq!(
///     ryuu::Formatter::format_finite_f32(1.234_f32).as_str(),
///     "1.234"
/// );
/// ```
pub struct Formatter;

#[derive(Clone, Copy)]
/// The formatted text of a floating point number.
///
/// This implements `AsRef<str>` and `ops::Deref<Target = str>`.
pub struct Formatted {
    /// The inner bytes, maybe initialized.
    bytes: [MaybeUninit<u8>; BUFFER_LEN],

    /// The type of the formatted number, which indicates whether it is an
    /// integer, has a decimal point, or is in exponent form.
    meta: FormattedMeta,

    /// The offset of all bytes that have been initialized in `bytes`.
    initialized: usize,
}

impl fmt::Debug for Formatted {
    /// Prints the string representation of the last formatted floating point
    /// number.
    ///
    /// # Panics
    ///
    /// This method panics if no floating point number has been formatted yet.
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Formatted")
            .field("bytes", &self.as_str())
            .field("meta", &self.meta)
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl AsRef<str> for Formatted {
    /// Returns a reference to the string representation of the last formatted
    /// floating point number.
    ///
    /// # Panics
    ///
    /// This method panics if no floating point number has been formatted yet.
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ops::Deref for Formatted {
    type Target = str;

    /// Returns a reference to the string representation of the last formatted
    /// floating point number.
    ///
    /// # Panics
    ///
    /// This method panics if no floating point number has been formatted yet.
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Formatted {
    #[inline]
    /// Returns the formatted text.
    pub const fn as_str(&self) -> &str {
        #[allow(unsafe_code)]
        // SAFETY: The `bytes` array is guaranteed to be valid ASCII.
        unsafe {
            str::from_utf8_unchecked(self.as_bytes())
        }
    }

    #[inline]
    /// Returns the formatted text bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        #[allow(unsafe_code)]
        // SAFETY: The `bytes` array is guaranteed to be initialized.
        unsafe {
            slice::from_raw_parts(self.bytes.as_ptr().cast::<u8>(), self.initialized)
        }
    }

    #[inline]
    /// Returns the formatted text with a fixed number of decimal places (on a
    /// best effort).
    ///
    /// ## Constant Parameters
    ///
    /// * `DECIMAL_PLACES` - The number of decimal places to include in the
    ///   output
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ryuu::Formatter;
    /// // For a number with more decimal places than requested
    /// let formatted = Formatter::format_finite_f64(3.14159);
    /// assert_eq!(formatted.as_str_fixed_dp::<2>(), "3.14");
    ///
    /// // For a number with fewer decimal places than requested
    /// let formatted = Formatter::format_finite_f64(3.1);
    /// assert_eq!(formatted.as_str_fixed_dp::<3>(), "3.1"); // Not padded to "3.100"
    ///
    /// // For a large number that is formatted in scientific notation form
    /// let formatted = Formatter::format_finite_f64(1.0123e16);
    /// assert_eq!(formatted.as_str_fixed_dp::<2>(), "1.0123e16"); // Keeps the same
    /// ```
    pub const fn as_str_fixed_dp<const DECIMAL_PLACES: usize>(&self) -> &str {
        match self.meta {
            FormattedMeta::Decimal { offset_decimal_point } => {
                let target_length = offset_decimal_point + DECIMAL_PLACES + 1;

                if offset_decimal_point + DECIMAL_PLACES < self.initialized {
                    unsafe {
                        let bytes = slice::from_raw_parts(self.bytes.as_ptr().cast::<u8>(), target_length);
                        str::from_utf8_unchecked(bytes)
                    }
                } else {
                    self.as_str()
                }
            }
            _ => self.as_str(),
        }
    }

    /// Copies the formatted text to the given buffer, with a fixed number of
    /// decimal places.
    ///
    /// This method provides precise control over the number of decimal places
    /// in the output, handling different formatting scenarios (decimal,
    /// exponent, and non-finite numbers) appropriately.
    ///
    /// ## Behavior by Format Type
    ///
    /// ### Decimal Format (e.g., "3.14159")
    /// - If `DECIMAL_PLACES` is 0, copies only the integer part (no decimal
    ///   point)
    /// - If `DECIMAL_PLACES` > 0, ensures exactly that many decimal places:
    ///   - Truncates if the original has more decimal places
    ///   - Pads with zeros if the original has fewer decimal places
    ///
    /// ### Exponent Format (e.g., "3.14e20")
    /// - Adjusts the decimal part while preserving the exponent
    /// - If `DECIMAL_PLACES` is 0, removes the decimal point entirely
    /// - Pads with zeros or truncates the decimal part as needed
    ///
    /// ### Non-finite Numbers (NaN, inf, -inf)
    /// - Ignores `DECIMAL_PLACES` and copies the string as-is
    ///
    /// ## Return Value
    ///
    /// Returns `Some(bytes_written)` on success, where `bytes_written` is the
    /// number of bytes written to the buffer. Returns `None` if the buffer is
    /// too small to hold the result.
    ///
    /// ## Buffer Requirements
    ///
    /// The buffer must be large enough to hold the result, [`BUFFER_LEN`] +
    /// `DECIMAL_PLACES` bytes is recommended.
    ///
    /// ## Constant Parameters
    ///
    /// * `DECIMAL_PLACES` - The exact number of decimal places to include in
    ///   the output.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use ryuu::Formatter;
    /// let mut buf = [0u8; 32 + 2];
    ///
    /// // Normal case
    /// let formatted = Formatter::format_finite_f64(3.14159);
    /// # let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3");
    /// let written = formatted.copy_to_bytes::<4>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"3.1415");
    /// # let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3.14159");
    /// let written = formatted.copy_to_bytes::<6>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"3.141590");
    ///
    /// let formatted = Formatter::format_finite_f64(-3.14159);
    /// # let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"-3");
    /// let written = formatted.copy_to_bytes::<4>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"-3.1415");
    /// # let written = formatted.copy_to_bytes::<5>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"-3.14159");
    /// let written = formatted.copy_to_bytes::<6>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"-3.141590");
    ///
    /// // For a large / small number that will be formatted in scientific notation form
    /// let formatted = Formatter::format_finite_f64(3e20);
    /// # let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3e20");
    /// let written = formatted.copy_to_bytes::<1>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"3.0e20");
    /// # let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3.00e20");
    ///
    /// let formatted = Formatter::format_finite_f64(3.14e20);
    /// # let written = formatted.copy_to_bytes::<0>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3e20");
    /// let written = formatted.copy_to_bytes::<1>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"3.1e20");
    /// # let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"3.14e20");
    /// let written = formatted.copy_to_bytes::<3>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"3.140e20");
    ///
    /// // For nonfinite numbers, the output is the same as `as_str()`
    /// let formatted = Formatter::format_f64(f64::NAN);
    /// let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    /// assert_eq!(&buf[..written], b"NaN");
    /// # let formatted = Formatter::format_f64(f64::INFINITY);
    /// # let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"inf");
    /// # let formatted = Formatter::format_f64(f64::NEG_INFINITY);
    /// # let written = formatted.copy_to_bytes::<2>(&mut buf).unwrap();
    /// # assert_eq!(&buf[..written], b"-inf");
    /// ```
    pub const fn copy_to_bytes<const DECIMAL_PLACES: usize>(&self, buf: &mut [u8]) -> Option<usize> {
        match self.meta {
            FormattedMeta::Decimal { offset_decimal_point } => {
                if DECIMAL_PLACES == 0 {
                    let Some((buf, _)) = buf.split_at_mut_checked(offset_decimal_point) else {
                        return None;
                    };

                    unsafe {
                        ptr::copy_nonoverlapping(
                            self.bytes.as_ptr().cast::<u8>(),
                            buf.as_mut_ptr(),
                            offset_decimal_point,
                        );
                    };

                    Some(offset_decimal_point)
                } else {
                    let target_length = offset_decimal_point + DECIMAL_PLACES + 1; // for the decimal point

                    let Some((buf, _)) = buf.split_at_mut_checked(target_length) else {
                        return None;
                    };

                    unsafe {
                        if target_length <= self.initialized {
                            ptr::copy_nonoverlapping(self.bytes.as_ptr().cast::<u8>(), buf.as_mut_ptr(), target_length);
                        } else {
                            // SAFETY: target_length > self.initialized
                            let (bytes, zeros) = buf.split_at_mut_unchecked(self.initialized);

                            ptr::copy_nonoverlapping(
                                self.bytes.as_ptr().cast::<u8>(),
                                bytes.as_mut_ptr(),
                                self.initialized,
                            );

                            zeros.as_mut_ptr().write_bytes(b'0', zeros.len());
                        }
                    }

                    Some(target_length)
                }
            }
            FormattedMeta::Exponent {
                offset_decimal_point,
                offset_exponent,
            } => {
                let target_decimal_part = if DECIMAL_PLACES == 0 { 0 } else { DECIMAL_PLACES + 1 };

                let (actual_integer_part, actual_decimal_part) = match offset_decimal_point {
                    Some(offset_decimal_point) => (offset_decimal_point, (offset_exponent - offset_decimal_point)),
                    None => (offset_exponent, 0),
                };

                let target_length = self.initialized - actual_decimal_part + target_decimal_part;

                let Some((buf, _)) = buf.split_at_mut_checked(target_length) else {
                    return None;
                };

                let (bytes_integer_part, bytes_decimal_part, bytes_exponent_part) = {
                    let bytes = self.as_bytes();
                    let (bytes_integer_part, bytes) = unsafe { bytes.split_at_unchecked(actual_integer_part) };
                    let (bytes_decimal_part, bytes_exponent_part) =
                        unsafe { bytes.split_at_unchecked(actual_decimal_part) };

                    (bytes_integer_part, bytes_decimal_part, bytes_exponent_part)
                };

                // The integer part
                let (buf_integer_part, buf) = unsafe { buf.split_at_mut_unchecked(bytes_integer_part.len()) };
                unsafe {
                    ptr::copy_nonoverlapping(
                        bytes_integer_part.as_ptr().cast::<u8>(),
                        buf_integer_part.as_mut_ptr(),
                        bytes_integer_part.len(),
                    );
                };

                // The decimal part
                let (buf_decimal_part, buf_exponent_part) = unsafe { buf.split_at_mut_unchecked(target_decimal_part) };
                if target_decimal_part > 0 {
                    match target_decimal_part.checked_sub(actual_decimal_part) {
                        Some(remaining) => {
                            unsafe {
                                if actual_decimal_part == 0 {
                                    // If there is no decimal part, we need to write the decimal point
                                    buf_decimal_part.as_mut_ptr().write(b'.');
                                    // Write zeros after the decimal point. Since we already wrote the decimal
                                    // point, we need to subtract 1 from
                                    // remaining to account for the decimal point.
                                    if remaining > 0 {
                                        buf_decimal_part.as_mut_ptr().offset(1).write_bytes(b'0', remaining - 1);
                                    }
                                } else {
                                    ptr::copy_nonoverlapping(
                                        bytes_decimal_part.as_ptr().cast::<u8>(),
                                        buf_decimal_part.as_mut_ptr(),
                                        actual_decimal_part,
                                    );

                                    buf_decimal_part
                                        .as_mut_ptr()
                                        .add(actual_decimal_part)
                                        .write_bytes(b'0', remaining);
                                }
                            };
                        }
                        None => {
                            unsafe {
                                ptr::copy_nonoverlapping(
                                    bytes_decimal_part.as_ptr().cast::<u8>(),
                                    buf_decimal_part.as_mut_ptr(),
                                    target_decimal_part,
                                );
                            };
                        }
                    }
                }

                // The exponent part.
                unsafe {
                    ptr::copy_nonoverlapping(
                        bytes_exponent_part.as_ptr(),
                        buf_exponent_part.as_mut_ptr(),
                        bytes_exponent_part.len(),
                    );
                };

                Some(target_length)
            }
            FormattedMeta::Nonfinite => {
                let Some((buf, _)) = buf.split_at_mut_checked(self.initialized) else {
                    return None;
                };

                let bytes = self.as_bytes();

                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr(), bytes.len()) };

                Some(self.initialized)
            }
        }
    }
}

impl Formatter {
    /// Print a floating point number into this buffer and return a reference to
    /// its string representation within the buffer.
    ///
    /// # Special cases
    ///
    /// This function formats NaN as the string "NaN", positive infinity as
    /// "inf", and negative infinity as "-inf" to match std::fmt.
    ///
    /// If your input is known to be finite, you may get better performance by
    /// calling the `format_finite` method instead of `format` to avoid the
    /// checks for special cases.
    ///
    /// Since const traits support is not yet stable, this function is not
    /// `const`: <https://rust-lang.github.io/rust-project-goals/2024h2/const-traits.html>.
    #[inline]
    pub fn format<F: Float>(f: F) -> Formatted {
        f.format()
    }

    #[inline]
    /// Const version of [`format`](Self::format), specifically for `f64`.
    pub const fn format_f64(d: f64) -> Formatted {
        if is_nonfinite_f64(d) {
            let nonfinite_formatted = format_nonfinite_f64(d);

            let mut bytes = [MaybeUninit::uninit(); BUFFER_LEN];

            unsafe {
                ptr::copy_nonoverlapping(
                    nonfinite_formatted.as_ptr(),
                    bytes.as_mut_ptr().cast::<u8>(),
                    nonfinite_formatted.len(),
                );
            };

            Formatted {
                bytes,
                meta: FormattedMeta::Nonfinite,
                initialized: nonfinite_formatted.len(),
            }
        } else {
            Self::format_finite_f64(d)
        }
    }

    #[inline]
    /// Const version of [`format`](Self::format), specifically for `f32`.
    pub const fn format_f32(f: f32) -> Formatted {
        if is_nonfinite_f32(f) {
            let nonfinite_formatted = format_nonfinite_f32(f);

            let mut bytes = [MaybeUninit::uninit(); BUFFER_LEN];

            unsafe {
                ptr::copy_nonoverlapping(
                    nonfinite_formatted.as_ptr(),
                    bytes.as_mut_ptr().cast::<u8>(),
                    nonfinite_formatted.len(),
                );
            };

            Formatted {
                bytes,
                meta: FormattedMeta::Nonfinite,
                initialized: nonfinite_formatted.len(),
            }
        } else {
            Self::format_finite_f32(f)
        }
    }

    /// Print a floating point number into this buffer and return a reference to
    /// its string representation within the buffer.
    ///
    /// # Special cases
    ///
    /// This function **does not** check for NaN or infinity. If the input
    /// number is not a finite float, the printed representation will be some
    /// correctly formatted but unspecified numerical value.
    ///
    /// Please check [`is_finite`] yourself before calling this function, or
    /// check [`is_nan`] and [`is_infinite`] and handle those cases yourself.
    ///
    /// [`is_finite`]: f64::is_finite
    /// [`is_nan`]: f64::is_nan
    /// [`is_infinite`]: f64::is_infinite
    #[inline]
    pub const fn format_finite_f64(d: f64) -> Formatted {
        let mut bytes = [MaybeUninit::uninit(); BUFFER_LEN];

        // Do format
        let offset_full = unsafe { raw::format64_spec(d, bytes.as_mut_ptr().cast::<u8>()) };

        debug_assert!(offset_full.initialized <= BUFFER_LEN);

        Formatted {
            bytes,
            meta: offset_full.meta,
            initialized: offset_full.initialized,
        }
    }

    #[inline]
    /// `f32` version of [`format_finite_f64`](Self::format_finite_f64).
    pub const fn format_finite_f32(f: f32) -> Formatted {
        let mut bytes = [MaybeUninit::uninit(); BUFFER_LEN];

        // Do format
        let offset_full = unsafe { raw::format32_spec(f, bytes.as_mut_ptr().cast::<u8>()) };

        debug_assert!(offset_full.initialized <= BUFFER_LEN);

        Formatted {
            bytes,
            meta: offset_full.meta,
            initialized: offset_full.initialized,
        }
    }
}

#[allow(private_bounds)]
/// A floating point number, f32 or f64, that can be formatted to text.
///
/// This trait is sealed and cannot be implemented for types outside of the
/// `ryu` crate.
pub trait Float: Sealed {}

impl Float for f32 {}
impl Float for f64 {}

trait Sealed: Copy {
    fn format(self) -> Formatted;
}

impl Sealed for f32 {
    #[inline]
    fn format(self) -> Formatted {
        Formatter::format_f32(self)
    }
}

impl Sealed for f64 {
    #[inline]
    fn format(self) -> Formatted {
        Formatter::format_f64(self)
    }
}

// === nonfinite float helpers ===

const NAN: &str = "NaN";
const INFINITY: &str = "inf";
const NEG_INFINITY: &str = "-inf";

#[inline]
const fn is_nonfinite_f32(f: f32) -> bool {
    const EXP_MASK: u32 = 0x7f800000;
    let bits = f.to_bits();
    bits & EXP_MASK == EXP_MASK
}

#[inline]
const fn is_nonfinite_f64(d: f64) -> bool {
    const EXP_MASK: u64 = 0x7ff0000000000000;
    let bits = d.to_bits();
    bits & EXP_MASK == EXP_MASK
}

#[cold]
#[inline]
const fn format_nonfinite_f64(d: f64) -> &'static str {
    const MANTISSA_MASK: u64 = 0x000fffffffffffff;
    const SIGN_MASK: u64 = 0x8000000000000000;
    let bits = d.to_bits();
    if bits & MANTISSA_MASK != 0 {
        NAN
    } else if bits & SIGN_MASK != 0 {
        NEG_INFINITY
    } else {
        INFINITY
    }
}

#[cold]
#[inline]
const fn format_nonfinite_f32(f: f32) -> &'static str {
    const MANTISSA_MASK: u32 = 0x007fffff;
    const SIGN_MASK: u32 = 0x80000000;
    let bits = f.to_bits();
    if bits & MANTISSA_MASK != 0 {
        NAN
    } else if bits & SIGN_MASK != 0 {
        NEG_INFINITY
    } else {
        INFINITY
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    mod test_copy_to_bytes {
        include!("../unittests/buffer_copy_to_bytes.rs");
    }
}
