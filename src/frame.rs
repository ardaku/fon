// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Frame (interleaved sample) types

use crate::chan::Channel;
use crate::ops::Ops;
use core::{
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
};

/// Frame - A number of interleaved sample [channel]s.
///
/// [channel]: crate::chan::Channel
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frame<Chan: Channel, const CH: usize>(pub(crate) [Chan; CH]);

impl<Chan: Channel, const CH: usize> Default for Frame<Chan, CH> {
    fn default() -> Self {
        Frame([Chan::default(); CH])
    }
}

impl<Chan: Channel, const CH: usize> Frame<Chan, CH>
where
    Self: Ops<Chan>,
{
    /// Mix a panned channel into this audio frame.
    ///
    /// 1.0/0.0 is straight ahead, 0.25 is right, 0.5 is back, and 0.75 is left.
    /// The algorithm used is "Constant Power Panning".
    #[inline(always)]
    pub fn pan<C: Channel + Into<Chan>>(&mut self, channel: C, angle: f32) {
        Ops::pan(self, channel.into(), angle)
    }

    /// Apply gain to the channel.  This function may introduce hard clipping
    /// distortion if `gain` is greater than 1.
    #[inline(always)]
    pub fn gain(&mut self, gain: f32) {
        for x in self.0.iter_mut() {
            *x = (x.to_f32() * gain).into();
        }
    }

    /// Apply linear interpolation with another frame.
    #[inline(always)]
    pub fn lerp(&mut self, rhs: Self, t: f32) {
        for (out, rhs) in self.0.iter_mut().zip(rhs.channels().iter()) {
            *out = out.lerp(*rhs, t.into());
        }
    }

    /// Convert an audio Frame to another format.
    #[inline(always)]
    pub fn to<C: Channel + From<Chan>, const N: usize>(self) -> Frame<C, N>
    where
        Frame<C, N>: Ops<C>,
    {
        Ops::to(self)
    }
}

impl<Chan: Channel> Frame<Chan, 1> {
    /// Create a new mono interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(mono: Chan) -> Self {
        Self([mono])
    }
}

impl<Chan: Channel> Frame<Chan, 2> {
    /// Create a new stereo interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan) -> Self {
        Self([left, right])
    }
}

impl<Chan: Channel> Frame<Chan, 3> {
    /// Create a new surround 3.0 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan) -> Self {
        Self([left, right, center])
    }
}

impl<Chan: Channel> Frame<Chan, 4> {
    /// Create a new surround 4.0 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(
        left: Chan,
        right: Chan,
        back_left: Chan,
        back_right: Chan,
    ) -> Self {
        Self([left, right, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 5> {
    /// Create a new surround 5.0 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(
        left: Chan,
        right: Chan,
        center: Chan,
        back_left: Chan,
        back_right: Chan,
    ) -> Self {
        Self([left, right, center, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 6> {
    /// Create a new surround 5.1 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(
        left: Chan,
        right: Chan,
        center: Chan,
        lfe: Chan,
        back_left: Chan,
        back_right: Chan,
    ) -> Self {
        Self([left, right, center, lfe, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 7> {
    /// Create a new surround 6.1 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(
        left: Chan,
        right: Chan,
        center: Chan,
        lfe: Chan,
        back: Chan,
        side_left: Chan,
        side_right: Chan,
    ) -> Self {
        Self([left, right, center, lfe, back, side_left, side_right])
    }
}

impl<Chan: Channel> Frame<Chan, 8> {
    /// Create a new surround 7.1 interleaved audio frame from channel(s).
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        left: Chan,
        right: Chan,
        center: Chan,
        lfe: Chan,
        back_left: Chan,
        back_right: Chan,
        side_left: Chan,
        side_right: Chan,
    ) -> Self {
        Self([
            left, right, center, lfe, back_left, back_right, side_left,
            side_right,
        ])
    }
}

impl<Chan: Channel, const CH: usize> Frame<Chan, CH> {
    /// Get the channels contained by this frame.
    #[inline(always)]
    pub fn channels(&self) -> &[Chan; CH] {
        &self.0
    }

    /// Get a mutable reference to the channels contained by this frame.
    #[inline(always)]
    pub fn channels_mut(&mut self) -> &mut [Chan; CH] {
        &mut self.0
    }
}

impl<Chan: Channel, const CH: usize> From<f32> for Frame<Chan, CH> {
    fn from(rhs: f32) -> Self {
        Frame([Chan::from(rhs); CH])
    }
}

impl<Chan: Channel, const CH: usize> Add for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.channels().iter()) {
            *a = *a + *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Sub for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.channels().iter()) {
            *a = *a - *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Mul for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.channels().iter()) {
            *a = *a * *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Neg for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for chan in self.0.iter_mut() {
            *chan = -*chan;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Iterator for Frame<Chan, CH> {
    type Item = Self;

    #[inline(always)]
    fn next(&mut self) -> Option<Self> {
        Some(*self)
    }
}

impl<Chan: Channel, const CH: usize> crate::Stream<Chan, CH>
    for Frame<Chan, CH>
{
    #[inline(always)]
    fn sample_rate(&self) -> Option<u32> {
        None
    }

    #[inline(always)]
    fn len(&self) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn set_sample_rate<R: Into<f64>>(&mut self, _: R) {}
}
