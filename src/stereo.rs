// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Stereo speaker configuration and types.

use crate::{
    chan::{Ch16, Ch24, Ch32, Channel},
    Frame,
};
use core::ops::{
    Add, Mul, Neg, Sub,
};

/// Stereo audio format (Audio [`Frame`](crate::frame::Frame) containing a left
/// and right [`Channel`](crate::chan::Channel)).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Stereo<C: Channel> {
    channels: [C; 2],
}

impl<C: Channel> Stereo<C> {
    /// Create a two-channel Sample.
    #[inline(always)]
    pub fn new<H>(one: H, two: H) -> Self
    where
        C: From<H>,
    {
        let channels = [C::from(one), C::from(two)];
        Self { channels }
    }
}

impl<C: Channel> Frame for Stereo<C> {
    const CONFIG: &'static [f64] = &[-0.5, 0.5];

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
        Self::new::<C>(ch[0], ch[1])
    }
}

impl<C: Channel> Add for Stereo<C> {
    type Output = Stereo<C>;

    #[inline(always)]
    fn add(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a + *b;
        }
        self
    }
}

impl<C: Channel> Sub for Stereo<C> {
    type Output = Stereo<C>;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a - *b;
        }
        self
    }
}

impl<C: Channel> Mul for Stereo<C> {
    type Output = Stereo<C>;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in self.channels.iter_mut().zip(other.channels.iter()) {
            *a = *a * *b;
        }
        self
    }
}

impl<C: Channel> Neg for Stereo<C> {
    type Output = Stereo<C>;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for chan in self.channels.iter_mut() {
            *chan = -*chan;
        }
        self
    }
}

impl<C: Channel> Iterator for Stereo<C> {
    type Item = Self;

    #[inline(always)]
    fn next(&mut self) -> Option<Self> {
        Some(*self)
    }
}

/// Stereo [16-bit PCM](crate::chan::Ch16) format.
pub type Stereo16 = Stereo<Ch16>;
/// Stereo [24-bit PCM](crate::chan::Ch24) format.
pub type Stereo24 = Stereo<Ch24>;
/// Stereo [32-bit Floating Point](crate::chan::Ch32) format.
pub type Stereo32 = Stereo<Ch32>;
