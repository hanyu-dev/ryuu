// Translated from C to Rust. The original C code can be found at
// https://github.com/ulfjack/ryu and carries the following license:
//
// Copyright 2018 Ulf Adams
//
// The contents of this file may be used under the terms of the Apache License,
// Version 2.0.
//
//    (See accompanying file LICENSE-Apache or copy at
//     http://www.apache.org/licenses/LICENSE-2.0)
//
// Alternatively, the contents of this file may be used under the terms of
// the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE-Boost or copy at
//     https://www.boost.org/LICENSE_1_0.txt)
//
// Unless required by applicable law or agreed to in writing, this software
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.

use crate::d2s_full_table::{DOUBLE_POW5_INV_SPLIT, DOUBLE_POW5_SPLIT};
use crate::d2s_small_table::{compute_inv_pow5, compute_pow5};

#[test]
fn test_compute_pow5() {
    for (i, entry) in DOUBLE_POW5_SPLIT.iter().enumerate() {
        assert_eq!(*entry, unsafe { compute_pow5(i as u32) }, "entry {i}");
    }
}

#[test]
fn test_compute_inv_pow5() {
    for (i, entry) in DOUBLE_POW5_INV_SPLIT[..292].iter().enumerate() {
        assert_eq!(*entry, unsafe { compute_inv_pow5(i as u32) }, "entry {i}");
    }
}
