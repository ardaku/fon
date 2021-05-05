// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Sample types

use crate::{chan::Channel};
use core::{
    fmt::Debug,
    ops::{
        Add, Mul, Neg, Sub,
    },
};

/// Frame - A number of interleaved sample [channel]s.
///
/// [channel]: crate::chan::Channel
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frame<Chan: Channel, const CH: usize>([Chan; CH]);

impl<Chan: Channel, const CH: usize> Default for Frame<Chan, CH> {
    fn default() -> Self {
        Frame([Chan::default(); CH])
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
    pub fn new(left: Chan, right: Chan, back_left: Chan, back_right: Chan) -> Self {
        Self([left, right, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 5> {
    /// Create a new surround 5.0 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan, back_left: Chan, back_right: Chan) -> Self {
        Self([left, right, center, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 6> {
    /// Create a new surround 5.1 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan, lfe: Chan, back_left: Chan, back_right: Chan) -> Self {
        Self([left, right, center, lfe, back_left, back_right])
    }
}

impl<Chan: Channel> Frame<Chan, 7> {
    /// Create a new surround 6.1 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan, lfe: Chan, back: Chan, side_left: Chan, side_right: Chan) -> Self {
        Self([left, right, center, lfe, back, side_left, side_right])
    }
}

impl<Chan: Channel> Frame<Chan, 8> {
    /// Create a new surround 7.1 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan, lfe: Chan, back_left: Chan, back_right: Chan, side_left: Chan, side_right: Chan) -> Self {
        Self([left, right, center, lfe, back_left, back_right, side_left, side_right])
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

    /// Linear interpolation.
    #[inline(always)]
    pub fn lerp(&self, rhs: Self, t: Self) -> Self {
        let mut out = Self::default();
        let main = out.channels_mut().iter_mut().zip(self.channels().iter());
        let other = rhs.channels().iter().zip(t.channels().iter());
        for ((out, this), (rhs, t)) in main.zip(other) {
            *out = this.lerp(*rhs, *t);
        }
        out
    }

    /// Convert an audio frame to another format.
    #[inline(always)]
    pub fn convert<C: Channel, const N: usize>(self) -> Frame<C, N>
        where C: From<Chan>
    {
        let mut out = Frame::<C, N>::default();

        match (CH, N) {
            (x, y) if x == y => {
                for (o, i) in out.channels_mut().iter_mut().zip(self.channels().iter()) {
                    *o = C::from(*i);
                }
            }
            // FIXME: Higher quality surround conversions for rest of match
            // statement.  What are better algorithms that people use?
            (1, _) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[0]);
            }
            (2, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.5)
                    + C::from(self.channels()[1]) * C::from(0.5);
            }
            (2, _) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
            }
            (3, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
            }
            (3, 2) | (3, 4) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
            }
            (3, _) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
            }
            (4, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.25)
                    + C::from(self.channels()[1]) * C::from(0.25)
                    + C::from(self.channels()[2]) * C::from(0.25)
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (4, 2) | (4, 3) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.5)
                    + C::from(self.channels()[2]) * C::from(0.5);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.5);
            }
            (4, 5) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[3] = C::from(self.channels()[2]);
                out.channels_mut()[4] = C::from(self.channels()[3]);
            }
            (4, 6) | (4, 8) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[4] = C::from(self.channels()[2]);
                out.channels_mut()[5] = C::from(self.channels()[3]);
            }
            (4, 7) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[4] =
                    C::from(self.channels()[2]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.5);
                out.channels_mut()[5] = 
                    C::from(self.channels()[0]) * C::from(0.5)
                    + C::from(self.channels()[2]) * C::from(0.5);
                out.channels_mut()[6] = 
                    C::from(self.channels()[1]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.5);
            }
            (5, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.2)
                    + C::from(self.channels()[1]) * C::from(0.2)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.2);
            }
            (5, 2) | (5, 3) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.4)
                    + C::from(self.channels()[4]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2);
            }
            (5, 4) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
                out.channels_mut()[2] = C::from(self.channels()[3]);
                out.channels_mut()[3] = C::from(self.channels()[4]);
            }
            (5, 6) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[4] = C::from(self.channels()[3]);
                out.channels_mut()[5] = C::from(self.channels()[4]);
            }
            (5, 7) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[4] =
                    C::from(self.channels()[3]) * C::from(0.5)
                    + C::from(self.channels()[4]) * C::from(0.5);
                out.channels_mut()[5] = C::from(self.channels()[3]);
                out.channels_mut()[6] = C::from(self.channels()[4]);
            }
            (5, 8) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[4] = C::from(self.channels()[3]);
                out.channels_mut()[5] = C::from(self.channels()[4]);
            }
            (6, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.2)
                    + C::from(self.channels()[1]) * C::from(0.2)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(0.2);
            }
            (6, 2) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.4)
                    + C::from(self.channels()[4]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.4);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.4)
                    + C::from(self.channels()[5]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.4);
            }
            (6, 3) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.5)
                    + C::from(self.channels()[4]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.5)
                    + C::from(self.channels()[5]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
            }
            (6, 4) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[2] = C::from(self.channels()[4])
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[3] = C::from(self.channels()[5])
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (6, 5) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) 
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[3] = C::from(self.channels()[4])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[4] = C::from(self.channels()[5])
                    + C::from(self.channels()[3]) * C::from(0.2);
            }
            (6, 7) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[5] = C::from(self.channels()[4]);
                out.channels_mut()[6] = C::from(self.channels()[5]);
            }
            (6, 8) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] = C::from(self.channels()[4]);
                out.channels_mut()[5] = C::from(self.channels()[5]);
            }
            (7, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 6.0);
            }
            (7, 2) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
            }
            (7, 3) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(0.5);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.5);
                out.channels_mut()[2] =
                    C::from(self.channels()[2]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.5);
            }
            (7, 4) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[2] =
                    C::from(self.channels()[5]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[3] =
                    C::from(self.channels()[6]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (7, 5) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[1] =
                    C::from(self.channels()[1])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[2] =
                    C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[3] =
                    C::from(self.channels()[5]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[4] =
                    C::from(self.channels()[6]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2);
            }
            (7, 6) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] =
                    C::from(self.channels()[6]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0);
                out.channels_mut()[5] =
                    C::from(self.channels()[6]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0);
            }
            (7, 8) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] = C::from(self.channels()[5]);
                out.channels_mut()[5] = C::from(self.channels()[6]);
                out.channels_mut()[6] =
                    C::from(self.channels()[4]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
                out.channels_mut()[7] =
                    C::from(self.channels()[4]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
            }
            (8, 1) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(1.0 / 7.0);
            }
            (8, 2) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[6]) * C::from(2.0 / 7.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[7]) * C::from(2.0 / 7.0);
            }
            (8, 3) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
                out.channels_mut()[2] =
                    C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(0.2);
            }
            (8, 4) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
                out.channels_mut()[2] =
                    C::from(self.channels()[4]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[3] =
                    C::from(self.channels()[5]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
            }
            (8, 5) => {
                out.channels_mut()[0] =
                    C::from(self.channels()[0]) * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[1] =
                    C::from(self.channels()[1]) * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] =
                    C::from(self.channels()[4]) * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[4] =
                    C::from(self.channels()[5]) * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
            }
            (8, 6) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] =
                    C::from(self.channels()[2]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[3] =
                    C::from(self.channels()[3]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
                out.channels_mut()[4] =
                    C::from(self.channels()[4]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[5] =
                    C::from(self.channels()[5]) * C::from(2.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
            }
            (8, 7) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] =
                    C::from(self.channels()[4]) * C::from(0.5)
                    + C::from(self.channels()[5]) * C::from(0.5);
                out.channels_mut()[5] = C::from(self.channels()[6]);
                out.channels_mut()[6] = C::from(self.channels()[7]);
            }
            _ => unreachable!(),
        }
        out
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
        for (a, b) in self.channels_mut().iter_mut().zip(other.channels().iter()) {
            *a = *a + *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Sub for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in self.channels_mut().iter_mut().zip(other.channels().iter()) {
            *a = *a - *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Mul for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in self.channels_mut().iter_mut().zip(other.channels().iter()) {
            *a = *a * *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Neg for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for chan in self.channels_mut().iter_mut() {
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

impl<Chan: Channel, const CH: usize> crate::Stream<Chan, CH> for Frame<Chan, CH> {
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
