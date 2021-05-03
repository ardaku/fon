// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Surround Sound 5.1 speaker configuration and types.

use crate::{
    chan::{Ch16, Ch24, Ch32, Channel},
    Frame,
};
use core::ops::{
    Add, Mul, Neg, Sub,
};

/// Surround Sound 5.1 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, rear left, rear right, front right, center, and lfe
/// [`Channel`](crate::chan::Channel)).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Surround<C: Channel> {
    channels: [C; 6],
}

impl<C: Channel> Surround<C> {
    /// Create a one-channel Sample.
    #[inline(always)]
    pub fn new<H>(one: H, two: H, three: H, four: H, five: H, six: H) -> Self
    where
        C: From<H>,
    {
        let channels = [
            C::from(one),
            C::from(two),
            C::from(three),
            C::from(four),
            C::from(five),
            C::from(six),
        ];
        Self { channels }
    }
}

impl<C: Channel> Frame for Surround<C> {
    const CONFIG: &'static [f64] = &[
        -2.0 / 3.0, // Rear Left
        -1.0 / 3.0, // Front Left
        0.0 / 3.0,  // Center
        1.0 / 3.0,  // Front Right
        2.0 / 3.0,  // Rear Right
    ];

    type Chan = C;

    #[inline(always)]
    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    #[inline(always)]
    fn from_channels(ch: &[Self::Chan]) -> Self {
        Self::new::<C>(ch[0], ch[1], ch[2], ch[3], ch[4], ch[5])
    }
}

impl<C: Channel> Add for Surround<C> {
    type Output = Surround<C>;

    #[inline(always)]
    fn add(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a + *b;
        }
        self
    }
}

impl<C: Channel> Sub for Surround<C> {
    type Output = Surround<C>;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a - *b;
        }
        self
    }
}

impl<C: Channel> Mul for Surround<C> {
    type Output = Surround<C>;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a * *b;
        }
        self
    }
}

impl<C: Channel> Neg for Surround<C> {
    type Output = Surround<C>;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for chan in self.channels.iter_mut() {
            *chan = -*chan;
        }
        self
    }
}

impl<C: Channel> Iterator for Surround<C> {
    type Item = Self;

    #[inline(always)]
    fn next(&mut self) -> Option<Self> {
        Some(*self)
    }
}

/// 5.1 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround16 = Surround<Ch16>;
/// 5.1 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround24 = Surround<Ch24>;
/// 5.1 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround32 = Surround<Ch32>;
