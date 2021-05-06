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

use crate::chan::{Ch16, Ch24, Ch32, Ch64, Channel};
use core::{
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, _x: f32) -> Self {
        let mut out = Self::default();
        // Do constant power panning.
        out.channels_mut()[0] = chan;
        out
    }
}

impl<Chan: Channel> Frame<Chan, 2> {
    /// Create a new stereo interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan) -> Self {
        Self([left, right])
    }

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        // Convert to radians, left is now at 0.
        let x = (x + 0.25) * FRAC_PI_2;
        // Pan distance
        out.channels_mut()[0] = chan * x.cos().into();
        out.channels_mut()[1] = chan * x.sin().into();

        out
    }
}

impl<Chan: Channel> Frame<Chan, 3> {
    /// Create a new surround 3.0 interleaved audio frame from channel(s).
    #[inline(always)]
    pub fn new(left: Chan, right: Chan, center: Chan) -> Self {
        Self([left, right, center])
    }

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        // All nearness distances are 1/4
        match (x.fract() + 1.0).fract() {
            // Center-Right Speakers
            x if x < 0.25 => {
                let x = 4.0 * x * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Right-Center Speakers
            x if x < 0.5 => {
                let x = 4.0 * (x - 0.25) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
            // Center-Left Speakers
            x if x < 0.75 => {
                let x = 4.0 * (x - 0.50) * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
            // Left-Center Speakers
            x => {
                let x = 4.0 * (x - 0.75) * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
        }

        out
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        // Make 0 be Front Left Speaker
        match (x.fract() + 1.0 + 1.0 / 12.0).fract() {
            // Front Left - Front Right Speakers (60° slice)
            x if x < 60.0 / 360.0 => {
                let x = (360.0 / 60.0) * x * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 140.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 60.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[3] = chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 280.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 140.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[3] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x => {
                let x = (360.0 / 80.0) * (x - 280.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
        }

        out
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[4] = chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[4] = chan * x.cos().into();
                out.channels_mut()[3] = chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[3] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
        }

        out
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[5] = chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[5] = chan * x.cos().into();
                out.channels_mut()[4] = chan * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[4] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
        }

        out
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[6] = chan * x.sin().into();
            }
            // Side Right - Back Speakers (90° slice)
            x if x < 180.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[6] = chan * x.cos().into();
                out.channels_mut()[4] = chan * x.sin().into();
            }
            // Back - Side Left Speakers (90° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 180.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[4] = chan * x.cos().into();
                out.channels_mut()[5] = chan * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[5] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
        }

        out
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

    /// Create frame from a single channel panned `x` rotations from front
    /// center.
    #[inline(always)]
    pub fn pan(chan: Chan, x: f32) -> Self {
        use std::f32::consts::FRAC_PI_2;

        let mut out = Self::default();
        // Do constant power panning.

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                out.channels_mut()[2] = chan * x.cos().into();
                out.channels_mut()[1] = chan * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[1] = chan * x.cos().into();
                out.channels_mut()[7] = chan * x.sin().into();
            }
            // Side Right - Back Right Speakers (60° slice)
            x if x < 150.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[7] = chan * x.cos().into();
                out.channels_mut()[5] = chan * x.sin().into();
            }
            // Back Right - Back Left Speakers (60° slice)
            x if x < 210.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 150.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[5] = chan * x.cos().into();
                out.channels_mut()[4] = chan * x.sin().into();
            }
            // Back Left - Side Left Speakers (60° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 210.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[4] = chan * x.cos().into();
                out.channels_mut()[5] = chan * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[5] = chan * x.cos().into();
                out.channels_mut()[0] = chan * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                out.channels_mut()[0] = chan * x.cos().into();
                out.channels_mut()[2] = chan * x.sin().into();
            }
        }

        out
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
    where
        C: From<Chan>,
    {
        let mut out = Frame::<C, N>::default();

        match (CH, N) {
            (x, y) if x == y => {
                for (o, i) in
                    out.channels_mut().iter_mut().zip(self.channels().iter())
                {
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
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.5)
                    + C::from(self.channels()[1]) * C::from(0.5);
            }
            (2, _) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
            }
            (3, 1) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(1.0 / 3.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
            }
            (3, 2) | (3, 4) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
            }
            (3, _) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
            }
            (4, 1) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.25)
                    + C::from(self.channels()[1]) * C::from(0.25)
                    + C::from(self.channels()[2]) * C::from(0.25)
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (4, 2) | (4, 3) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.5)
                    + C::from(self.channels()[2]) * C::from(0.5);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.5)
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
                out.channels_mut()[4] = C::from(self.channels()[2])
                    * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.5);
                out.channels_mut()[5] = C::from(self.channels()[0])
                    * C::from(0.5)
                    + C::from(self.channels()[2]) * C::from(0.5);
                out.channels_mut()[6] = C::from(self.channels()[1])
                    * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.5);
            }
            (5, 1) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.2)
                    + C::from(self.channels()[1]) * C::from(0.2)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.2);
            }
            (5, 2) | (5, 3) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.4)
                    + C::from(self.channels()[4]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2);
            }
            (5, 4) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(2.0 / 3.0)
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
                out.channels_mut()[4] = C::from(self.channels()[3])
                    * C::from(0.5)
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
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.2)
                    + C::from(self.channels()[1]) * C::from(0.2)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(0.2);
            }
            (6, 2) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.4)
                    + C::from(self.channels()[4]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.4);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.4)
                    + C::from(self.channels()[5]) * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.2)
                    + C::from(self.channels()[3]) * C::from(0.4);
            }
            (6, 3) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.5)
                    + C::from(self.channels()[4]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.5)
                    + C::from(self.channels()[5]) * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(1.0 / 3.0);
            }
            (6, 4) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[2] = C::from(self.channels()[4])
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[3] = C::from(self.channels()[5])
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (6, 5) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[1] = C::from(self.channels()[1])
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
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(1.0 / 6.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 6.0);
            }
            (7, 2) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(1.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 6.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
            }
            (7, 3) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(0.5);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.5);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    * C::from(0.5)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(0.5);
            }
            (7, 4) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[2] = C::from(self.channels()[5])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
                out.channels_mut()[3] = C::from(self.channels()[6])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.25);
            }
            (7, 5) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[3] = C::from(self.channels()[5])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2);
                out.channels_mut()[4] = C::from(self.channels()[6])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2);
            }
            (7, 6) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] = C::from(self.channels()[6])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0);
                out.channels_mut()[5] = C::from(self.channels()[6])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0);
            }
            (7, 8) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] = C::from(self.channels()[5]);
                out.channels_mut()[5] = C::from(self.channels()[6]);
                out.channels_mut()[6] = C::from(self.channels()[4])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
                out.channels_mut()[7] = C::from(self.channels()[4])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0);
            }
            (8, 1) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(1.0 / 7.0)
                    + C::from(self.channels()[1]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[4]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[5]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(1.0 / 7.0);
            }
            (8, 2) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(2.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[6]) * C::from(2.0 / 7.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(2.0 / 7.0)
                    + C::from(self.channels()[2]) * C::from(1.0 / 7.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(2.0 / 7.0)
                    + C::from(self.channels()[7]) * C::from(2.0 / 7.0);
            }
            (8, 3) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[4]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(1.0 / 3.0)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[5]) * C::from(1.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    + C::from(self.channels()[3]) * C::from(0.2);
            }
            (8, 4) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
                out.channels_mut()[2] = C::from(self.channels()[4])
                    * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[3] = C::from(self.channels()[5])
                    * C::from(0.4)
                    + C::from(self.channels()[2]) * C::from(0.4)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
            }
            (8, 5) => {
                out.channels_mut()[0] = C::from(self.channels()[0])
                    * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[1] = C::from(self.channels()[1])
                    * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[4])
                    * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[6]) * C::from(0.2);
                out.channels_mut()[4] = C::from(self.channels()[5])
                    * C::from(0.8)
                    + C::from(self.channels()[3]) * C::from(0.2)
                    + C::from(self.channels()[7]) * C::from(0.2);
            }
            (8, 6) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[3] = C::from(self.channels()[3])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
                out.channels_mut()[4] = C::from(self.channels()[4])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[6]) * C::from(1.0 / 3.0);
                out.channels_mut()[5] = C::from(self.channels()[5])
                    * C::from(2.0 / 3.0)
                    + C::from(self.channels()[7]) * C::from(1.0 / 3.0);
            }
            (8, 7) => {
                out.channels_mut()[0] = C::from(self.channels()[0]);
                out.channels_mut()[1] = C::from(self.channels()[1]);
                out.channels_mut()[2] = C::from(self.channels()[2]);
                out.channels_mut()[3] = C::from(self.channels()[3]);
                out.channels_mut()[4] = C::from(self.channels()[4])
                    * C::from(0.5)
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
        for (a, b) in
            self.channels_mut().iter_mut().zip(other.channels().iter())
        {
            *a = *a + *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Sub for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in
            self.channels_mut().iter_mut().zip(other.channels().iter())
        {
            *a = *a - *b;
        }
        self
    }
}

impl<Chan: Channel, const CH: usize> Mul for Frame<Chan, CH> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in
            self.channels_mut().iter_mut().zip(other.channels().iter())
        {
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

/// Mono audio format (Audio [`Frame`](crate::frame::Frame) containing a single
/// [`Channel`](crate::chan::Channel)).
pub type Mono<Chan> = Frame<Chan, 1>;

/// Mono [16-bit PCM](crate::chan::Ch16) format.
pub type Mono16 = Mono<Ch16>;
/// Mono [24-bit Floating Point](crate::chan::Ch24) format.
pub type Mono24 = Mono<Ch24>;
/// Mono [32-bit Floating Point](crate::chan::Ch32) format.
pub type Mono32 = Mono<Ch32>;
/// Mono [64-bit Floating Point](crate::chan::Ch64) format.
pub type Mono64 = Mono<Ch64>;

/// Stereo audio format (Audio [`Frame`](crate::frame::Frame) containing a left
/// and right [`Channel`](crate::chan::Channel)).
pub type Stereo<Chan> = Frame<Chan, 2>;

/// Stereo [16-bit PCM](crate::chan::Ch16) format.
pub type Stereo16 = Stereo<Ch16>;
/// Stereo [24-bit PCM](crate::chan::Ch24) format.
pub type Stereo24 = Stereo<Ch24>;
/// Stereo [32-bit Floating Point](crate::chan::Ch32) format.
pub type Stereo32 = Stereo<Ch32>;
/// Stereo [64-bit Floating Point](crate::chan::Ch64) format.
pub type Stereo64 = Stereo<Ch64>;

/// Surround Sound 3.0 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front right, and center
/// [`Channel`](crate::chan::Channel)).
pub type Surround30<Chan> = Frame<Chan, 3>;

/// 3.0 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround30_16 = Surround30<Ch16>;
/// 3.0 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround30_24 = Surround30<Ch24>;
/// 3.0 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround30_32 = Surround30<Ch32>;
/// 3.0 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround30_64 = Surround30<Ch64>;

/// Surround Sound 4.0 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front right, and back left, back right
/// [`Channel`](crate::chan::Channel)).
pub type Surround40<Chan> = Frame<Chan, 4>;

/// 4.0 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround40_16 = Surround40<Ch16>;
/// 4.0 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround40_24 = Surround40<Ch24>;
/// 4.0 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround40_32 = Surround40<Ch32>;
/// 4.0 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround40_64 = Surround40<Ch64>;

/// Surround Sound 5.0 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front left, and center
/// [`Channel`](crate::chan::Channel)).
pub type Surround50<Chan> = Frame<Chan, 5>;

/// 5.0 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround50_16 = Surround40<Ch16>;
/// 5.0 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround50_24 = Surround40<Ch24>;
/// 5.0 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround50_32 = Surround40<Ch32>;
/// 5.0 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround50_64 = Surround40<Ch64>;

/// Surround Sound 5.1 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front right, center, lfe, back left, and back right
/// [`Channel`](crate::chan::Channel)).
pub type Surround51<Chan> = Frame<Chan, 6>;

/// 5.1 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround51_16 = Surround51<Ch16>;
/// 5.1 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround51_24 = Surround51<Ch24>;
/// 5.1 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround51_32 = Surround51<Ch32>;
/// 5.1 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround51_64 = Surround51<Ch64>;

/// Surround Sound 6.1 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front right, center, lfe, back, side left, and side
/// right [`Channel`](crate::chan::Channel)).
pub type Surround61<Chan> = Frame<Chan, 7>;

/// 6.1 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround61_16 = Surround61<Ch16>;
/// 6.1 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround61_24 = Surround61<Ch24>;
/// 6.1 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround61_32 = Surround61<Ch32>;
/// 6.1 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround61_64 = Surround61<Ch64>;

/// Surround Sound 7.1 audio format (Audio [`Frame`](crate::frame::Frame)
/// containing a front left, front right, center, lfe, back left, and back right
/// [`Channel`](crate::chan::Channel)).
pub type Surround71<Chan> = Frame<Chan, 8>;

/// 7.1 Surround [16-bit PCM](crate::chan::Ch16) format.
pub type Surround71_16 = Surround71<Ch16>;
/// 7.1 Surround [24-bit Floating Point](crate::chan::Ch24) format.
pub type Surround71_24 = Surround71<Ch24>;
/// 7.1 Surround [32-bit Floating Point](crate::chan::Ch32) format.
pub type Surround71_32 = Surround71<Ch32>;
/// 7.1 Surround [64-bit Floating Point](crate::chan::Ch64) format.
pub type Surround71_64 = Surround71<Ch64>;
