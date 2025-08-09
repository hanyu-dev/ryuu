#![doc = include_str!("../README.md")]
#![no_std]
#![doc(html_root_url = "https://docs.rs/ryu/1.0.20")]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::doc_markdown,
    clippy::expl_impl_clone_on_copy,
    clippy::if_not_else,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_imports
)]

mod common;
mod d2s;
#[cfg(any(test, not(feature = "small"), feature = "feat-exp-parse"))]
mod d2s_full_table;
mod d2s_intrinsics;
#[cfg(any(test, feature = "small", feature = "feat-exp-parse"))]
mod d2s_small_table;
mod digit_table;
mod f2s;
mod f2s_intrinsics;
pub mod format;
#[cfg(any(test, feature = "feat-exp-parse"))]
pub mod parse;
pub mod raw;

pub use crate::format::{Formatted, Formatter};
