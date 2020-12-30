// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

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
