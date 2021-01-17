// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Math not available on no_std.

#[inline(always)]
pub(crate) fn floorh_i16(input: f32) -> i16 {
    if input < 0.0 {
        (-ceilh(-input)) as i16
    } else {
        input as i16
    }
}

#[inline(always)]
pub(crate) fn floorh_i8(input: f32) -> i8 {
    if input < 0.0 {
        (-ceilh(-input)) as i8
    } else {
        input as i8
    }
}

#[inline(always)]
pub(crate) fn floor_i16(input: f64) -> i16 {
    if input < 0.0 {
        (-ceil(-input)) as i16
    } else {
        input as i16
    }
}

#[inline(always)]
pub(crate) fn floor_i8(input: f64) -> i8 {
    if input < 0.0 {
        (-ceil(-input)) as i8
    } else {
        input as i8
    }
}

// Only use for unsigned values.
#[inline(always)]
pub(crate) fn ceilh(input: f32) -> f32 {
    let fract = input % 1.0;
    let mut whole = input - fract;
    if fract > f32::EPSILON {
        whole += 1.0;
    }
    whole
}

// Only use for unsigned values.
#[inline(always)]
pub(crate) fn ceil(input: f64) -> f64 {
    let fract = input % 1.0;
    let mut whole = input - fract;
    if fract > f64::EPSILON {
        whole += 1.0;
    }
    whole
}

// Only use for unsigned values.
#[inline(always)]
pub(crate) fn ceil_usize(input: f64) -> usize {
    let fract = input % 1.0;
    let mut whole = input as usize;
    if fract > f64::EPSILON {
        whole += 1;
    }
    whole
}
