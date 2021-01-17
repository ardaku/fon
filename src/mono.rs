// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Mono speaker configuration and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8, Channel},
    Frame,
};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// Mono audio format (Audio [`Frame`](crate::frame::Frame) containing one
/// [`Channel`](crate::chan::Channel)).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Mono<C: Channel> {
    channels: [C; 1],
}

impl<C: Channel> Mono<C> {
    /// Create a one-channel audio [`Frame`](crate::frame::Frame).
    pub fn new<H>(one: H) -> Self
    where
        C: From<H>,
    {
        let channels = [C::from(one)];
        Self { channels }
    }
}

impl<C: Channel> Frame for Mono<C> {
    const CONFIG: &'static [f64] = &[0.0 /* one centered speaker */];

    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        Self::new::<C>(ch[0])
    }
}

impl<C: Channel> AddAssign for Mono<C> {
    fn add_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan += *ch;
        }
    }
}

impl<C: Channel> Add for Mono<C> {
    type Output = Mono<C>;

    fn add(mut self, other: Self) -> Self {
        self += other;
        self
    }
}

impl<C: Channel> SubAssign for Mono<C> {
    fn sub_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan -= *ch;
        }
    }
}

impl<C: Channel> Sub for Mono<C> {
    type Output = Mono<C>;

    fn sub(mut self, other: Self) -> Self {
        self -= other;
        self
    }
}

impl<C: Channel> MulAssign for Mono<C> {
    fn mul_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan *= *ch;
        }
    }
}

impl<C: Channel> Mul for Mono<C> {
    type Output = Mono<C>;

    fn mul(mut self, other: Self) -> Self {
        self *= other;
        self
    }
}

impl<C: Channel> DivAssign for Mono<C> {
    fn div_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan /= *ch;
        }
    }
}

impl<C: Channel> Div for Mono<C> {
    type Output = Mono<C>;

    fn div(mut self, other: Self) -> Self {
        self /= other;
        self
    }
}

impl<C: Channel> Neg for Mono<C> {
    type Output = Mono<C>;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for chan in self.channels.iter_mut() {
            *chan = -*chan;
        }
        self
    }
}

impl<C: Channel> Iterator for Mono<C> {
    type Item = Self;

    fn next(&mut self) -> Option<Self> {
        Some(*self)
    }
}

/// Mono [8-bit PCM](crate::chan::Ch8) format.
pub type Mono8 = Mono<Ch8>;
/// Mono [16-bit PCM](crate::chan::Ch16) format.
pub type Mono16 = Mono<Ch16>;
/// Mono [32-bit Floating Point](crate::chan::Ch32) format.
pub type Mono32 = Mono<Ch32>;
/// Mono [64-bit Floating Point](crate::chan::Ch64) format.
pub type Mono64 = Mono<Ch64>;
