// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Stereo speaker configuration and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8, Channel},
    Frame,
};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
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

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        Self::new::<C>(ch[0], ch[1])
    }
}

impl<C: Channel> AddAssign for Stereo<C> {
    fn add_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan += *ch;
        }
    }
}

impl<C: Channel> Add for Stereo<C> {
    type Output = Stereo<C>;

    fn add(mut self, other: Self) -> Self {
        self += other;
        self
    }
}

impl<C: Channel> SubAssign for Stereo<C> {
    fn sub_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan -= *ch;
        }
    }
}

impl<C: Channel> Sub for Stereo<C> {
    type Output = Stereo<C>;

    fn sub(mut self, other: Self) -> Self {
        self -= other;
        self
    }
}

impl<C: Channel> MulAssign for Stereo<C> {
    fn mul_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan *= *ch;
        }
    }
}

impl<C: Channel> Mul for Stereo<C> {
    type Output = Stereo<C>;

    fn mul(mut self, other: Self) -> Self {
        self *= other;
        self
    }
}

impl<C: Channel> DivAssign for Stereo<C> {
    fn div_assign(&mut self, other: Self) {
        for (chan, ch) in self.channels.iter_mut().zip(other.channels.iter()) {
            *chan /= *ch;
        }
    }
}

impl<C: Channel> Div for Stereo<C> {
    type Output = Stereo<C>;

    fn div(mut self, other: Self) -> Self {
        self /= other;
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

    fn next(&mut self) -> Option<Self> {
        Some(*self)
    }
}

/// Stereo [8-bit PCM](crate::chan::Ch8) format.
pub type Stereo8 = Stereo<Ch8>;
/// Stereo [16-bit PCM](crate::chan::Ch16) format.
pub type Stereo16 = Stereo<Ch16>;
/// Stereo [32-bit Floating Point](crate::chan::Ch32) format.
pub type Stereo32 = Stereo<Ch32>;
/// Stereo [64-bit Floating Point](crate::chan::Ch64) format.
pub type Stereo64 = Stereo<Ch64>;
