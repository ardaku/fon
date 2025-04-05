//! Frame (interleaved sample) types

use core::{
    f32::consts::FRAC_PI_2,
    fmt::Debug,
    ops::{Add, Mul, Neg, Sub},
};

use crate::chan::Channel;
#[cfg(not(test))]
use crate::math::Libm;

/// Frame - A number of interleaved [`Channel`]s
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frame<C: Channel, const COUNT: usize>([C; COUNT]);

impl<C: Channel, const COUNT: usize> Default for Frame<C, COUNT> {
    fn default() -> Self {
        Frame([C::default(); COUNT])
    }
}

impl<C: Channel, const COUNT: usize> Frame<C, COUNT> {
    /// Get a mutable slice of the samples in this frame.
    #[inline(always)]
    pub fn samples_mut(&mut self) -> &mut [C; COUNT] {
        &mut self.0
    }

    /// Get a slice of the samples in this frame.
    #[inline(always)]
    pub fn samples(&self) -> &[C; COUNT] {
        &self.0
    }

    /// Mix a panned channel into this audio frame.
    ///
    /// 1.0/0.0 is straight ahead, 0.25 is right, 0.5 is back, and 0.75 is left.
    /// The algorithm used is "Constant Power Panning".
    #[inline(always)]
    pub fn pan<S: Channel + Into<C>>(self, sample: S, angle: f32) -> Self {
        match COUNT {
            1 => self.pan_1(sample.into(), angle.rem_euclid(1.0)),
            2 => self.pan_2(sample.into(), angle.rem_euclid(1.0)),
            3 => self.pan_3(sample.into(), angle.rem_euclid(1.0)),
            4 => self.pan_4(sample.into(), angle.rem_euclid(1.0)),
            5 => self.pan_5(sample.into(), angle.rem_euclid(1.0)),
            6 => self.pan_6(sample.into(), angle.rem_euclid(1.0)),
            7 => self.pan_7(sample.into(), angle.rem_euclid(1.0)),
            8 => self.pan_8(sample.into(), angle.rem_euclid(1.0)),
            _ => unreachable!(),
        }
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
        for (out, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *out = out.lerp(*rhs, t.into());
        }
    }

    /// Convert an audio Frame to another format.
    #[inline(always)]
    pub fn to<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        match COUNT {
            1 => self.to_1(),
            2 => self.to_2(),
            3 => self.to_3(),
            4 => self.to_4(),
            5 => self.to_5(),
            6 => self.to_6(),
            7 => self.to_7(),
            8 => self.to_8(),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn pan_1(mut self, samp: C, _x: f32) -> Self {
        const MONO: usize = 0;

        self.0[MONO] += samp;

        self
    }

    #[inline(always)]
    fn pan_2(mut self, samp: C, x: f32) -> Self {
        const LEFT: usize = 0;
        const RIGHT: usize = 1;

        // Convert to radians, left is now at 0.
        let x = (x + 0.25) * core::f32::consts::PI;
        // Pan distance
        self.0[LEFT] += samp * x.cos().into();
        self.0[RIGHT] += samp * x.sin().into();

        self
    }

    #[inline(always)]
    fn pan_3(mut self, samp: C, x: f32) -> Self {
        const LEFT: usize = 0;
        const RIGHT: usize = 1;
        const CENTER: usize = 2;

        // All nearness distances are 1/4
        match (x.fract() + 1.0).fract() {
            // Center-Right Speakers
            x if x < 0.25 => {
                let x = 4.0 * x * FRAC_PI_2;
                self.0[CENTER] += samp * x.cos().into();
                self.0[RIGHT] += samp * x.sin().into();
            }
            // Right-Center Speakers
            x if x < 0.5 => {
                let x = 4.0 * (x - 0.25) * FRAC_PI_2;
                self.0[RIGHT] += samp * x.cos().into();
                self.0[CENTER] += samp * x.sin().into();
            }
            // Center-Left Speakers
            x if x < 0.75 => {
                let x = 4.0 * (x - 0.50) * FRAC_PI_2;
                self.0[CENTER] += samp * x.cos().into();
                self.0[LEFT] += samp * x.sin().into();
            }
            // Left-Center Speakers
            x => {
                let x = 4.0 * (x - 0.75) * FRAC_PI_2;
                self.0[LEFT] += samp * x.cos().into();
                self.0[CENTER] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn pan_4(mut self, samp: C, x: f32) -> Self {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const SURROUND_L: usize = 2;
        const SURROUND_R: usize = 3;

        // Make 0 be Front Left Speaker
        match (x.fract() + 1.0 + 1.0 / 12.0).fract() {
            // Front Left - Front Right Speakers (60° slice)
            x if x < 60.0 / 360.0 => {
                let x = (360.0 / 60.0) * x * FRAC_PI_2;
                self.0[FRONT_L] += samp * x.cos().into();
                self.0[FRONT_R] += samp * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 140.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 60.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_R] += samp * x.cos().into();
                self.0[SURROUND_R] += samp * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 280.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 140.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_R] += samp * x.cos().into();
                self.0[SURROUND_L] += samp * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x => {
                let x = (360.0 / 80.0) * (x - 280.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_L] += samp * x.cos().into();
                self.0[FRONT_L] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn pan_5(mut self, samp: C, x: f32) -> Self {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;
        const SURROUND_L: usize = 3;
        const SURROUND_R: usize = 4;

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self.0[FRONT] += samp * x.cos().into();
                self.0[FRONT_R] += samp * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_R] += samp * x.cos().into();
                self.0[SURROUND_R] += samp * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_R] += samp * x.cos().into();
                self.0[SURROUND_L] += samp * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_L] += samp * x.cos().into();
                self.0[FRONT_L] += samp * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_L] += samp * x.cos().into();
                self.0[FRONT] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn pan_6(mut self, samp: C, x: f32) -> Self {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;

        const SURROUND_L: usize = 4;
        const SURROUND_R: usize = 5;

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self.0[FRONT] += samp * x.cos().into();
                self.0[FRONT_R] += samp * x.sin().into();
            }
            // Front Right - Back Right Speakers (80° slice)
            x if x < 110.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_R] += samp * x.cos().into();
                self.0[SURROUND_R] += samp * x.sin().into();
            }
            // Back Right - Back Left Speakers (140° slice)
            x if x < 250.0 / 360.0 => {
                let x = (360.0 / 140.0) * (x - 110.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_R] += samp * x.cos().into();
                self.0[SURROUND_L] += samp * x.sin().into();
            }
            // Back Left - Front Left Speakers (80° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 80.0) * (x - 250.0 / 360.0) * FRAC_PI_2;
                self.0[SURROUND_L] += samp * x.cos().into();
                self.0[FRONT_L] += samp * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_L] += samp * x.cos().into();
                self.0[FRONT] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn pan_7(mut self, samp: C, x: f32) -> Self {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;

        const BACK: usize = 4;
        const LEFT: usize = 5;
        const RIGHT: usize = 6;

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self.0[FRONT] += samp * x.cos().into();
                self.0[FRONT_R] += samp * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_R] += samp * x.cos().into();
                self.0[RIGHT] += samp * x.sin().into();
            }
            // Side Right - Back Speakers (90° slice)
            x if x < 180.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                self.0[RIGHT] += samp * x.cos().into();
                self.0[BACK] += samp * x.sin().into();
            }
            // Back - Side Left Speakers (90° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 90.0) * (x - 180.0 / 360.0) * FRAC_PI_2;
                self.0[BACK] += samp * x.cos().into();
                self.0[LEFT] += samp * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                self.0[LEFT] += samp * x.cos().into();
                self.0[FRONT_L] += samp * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_L] += samp * x.cos().into();
                self.0[FRONT] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn pan_8(mut self, samp: C, x: f32) -> Self {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;

        const BACK_L: usize = 4;
        const BACK_R: usize = 5;
        const LEFT: usize = 6;
        const RIGHT: usize = 7;

        match (x.fract() + 1.0).fract() {
            // Front Center - Front Right Speakers (30° slice)
            x if x < 30.0 / 360.0 => {
                let x = (360.0 / 30.0) * x * FRAC_PI_2;
                self.0[FRONT] += samp * x.cos().into();
                self.0[FRONT_R] += samp * x.sin().into();
            }
            // Front Right - Side Right Speakers (60° slice)
            x if x < 90.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 30.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_R] += samp * x.cos().into();
                self.0[RIGHT] += samp * x.sin().into();
            }
            // Side Right - Back Right Speakers (60° slice)
            x if x < 150.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 90.0 / 360.0) * FRAC_PI_2;
                self.0[RIGHT] += samp * x.cos().into();
                self.0[BACK_R] += samp * x.sin().into();
            }
            // Back Right - Back Left Speakers (60° slice)
            x if x < 210.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 150.0 / 360.0) * FRAC_PI_2;
                self.0[BACK_R] += samp * x.cos().into();
                self.0[BACK_L] += samp * x.sin().into();
            }
            // Back Left - Side Left Speakers (60° slice)
            x if x < 270.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 210.0 / 360.0) * FRAC_PI_2;
                self.0[BACK_L] += samp * x.cos().into();
                self.0[LEFT] += samp * x.sin().into();
            }
            // Side Left - Front Left Speakers (60° slice)
            x if x < 330.0 / 360.0 => {
                let x = (360.0 / 60.0) * (x - 270.0 / 360.0) * FRAC_PI_2;
                self.0[LEFT] += samp * x.cos().into();
                self.0[FRONT_L] += samp * x.sin().into();
            }
            // Front Left - Center Speakers (30° slice)
            x => {
                let x = (360.0 / 30.0) * (x - 330.0 / 360.0) * FRAC_PI_2;
                self.0[FRONT_L] += samp * x.cos().into();
                self.0[FRONT] += samp * x.sin().into();
            }
        }

        self
    }

    #[inline(always)]
    fn to_1<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const MONO: usize = 0;

        let mut frame = Frame::<S, N>::default();
        let mono = self.0[MONO].into();
        frame.0[0] = mono;
        if N == 1 {
        } else {
            // Mono should always be mixed up to first two channels.
            frame.0[1] = mono;
        }
        frame
    }

    #[inline(always)]
    fn to_2<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const LEFT: usize = 0;
        const RIGHT: usize = 1;

        let mut frame = Frame::<S, N>::default();
        let left = self.0[LEFT].into();
        let right = self.0[RIGHT].into();
        if N == 1 {
            let half = 0.5.into();
            frame.0[0] = left * half + right * half;
        } else {
            // stereo should always be mixed up to first two channels.
            frame.0[0] = left;
            frame.0[1] = right;
        }
        frame
    }

    #[inline(always)]
    fn to_3<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const LEFT: usize = 0;
        const RIGHT: usize = 1;
        const CENTER: usize = 2;

        let mut frame = Frame::<S, N>::default();
        let left = self.0[LEFT].into();
        let right = self.0[RIGHT].into();
        let center = self.0[CENTER].into();
        match N {
            1 => {
                frame.0[0] = left * (1.0 / 3.0).into()
                    + right * (1.0 / 3.0).into()
                    + center * (1.0 / 3.0).into()
            }
            2 => {
                frame.0[0] =
                    left * (2.0 / 3.0).into() + center * (1.0 / 3.0).into();
                frame.0[1] =
                    right * (2.0 / 3.0).into() + center * (1.0 / 3.0).into();
            }
            4 => {
                frame.0[0] =
                    left * (2.0 / 3.0).into() + center * (1.0 / 3.0).into();
                frame.0[1] =
                    right * (2.0 / 3.0).into() + center * (1.0 / 3.0).into();
                frame.0[2] = frame.0[0];
                frame.0[3] = frame.0[1];
            }
            _ => {
                frame.0[0] = left;
                frame.0[1] = right;
                frame.0[2] = center;
            }
        }
        frame
    }

    #[inline(always)]
    fn to_4<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const SURROUND_L: usize = 2;
        const SURROUND_R: usize = 3;

        // Surround mix.
        let front_l = self.0[FRONT_L];
        let front_r = self.0[FRONT_R];
        let surround_l = self.0[SURROUND_L];
        let surround_r = self.0[SURROUND_R];
        // Amplitude reduction.
        let amplitude = (N as f32 / 4.0).min(1.0);
        Frame::<S, N>::default()
            .pan(front_l * amplitude.into(), -30.0 / 360.0)
            .pan(front_r * amplitude.into(), 30.0 / 360.0)
            .pan(surround_l * amplitude.into(), -110.0 / 360.0)
            .pan(surround_r * amplitude.into(), 110.0 / 360.0)
    }

    #[inline(always)]
    fn to_5<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;
        const SURROUND_L: usize = 3;
        const SURROUND_R: usize = 4;

        // Surround mix.
        let front_l = self.0[FRONT_L];
        let front_r = self.0[FRONT_R];
        let surround_l = self.0[SURROUND_L];
        let surround_r = self.0[SURROUND_R];
        let front = self.0[FRONT];
        // Amplitude reduction.
        let amplitude = (N as f32 / 5.0).min(1.0);
        Frame::<S, N>::default()
            .pan(front_l * amplitude.into(), -30.0 / 360.0)
            .pan(front_r * amplitude.into(), 30.0 / 360.0)
            .pan(surround_l * amplitude.into(), -110.0 / 360.0)
            .pan(surround_r * amplitude.into(), 110.0 / 360.0)
            .pan(front * amplitude.into(), 0.0)
    }

    #[inline(always)]
    fn to_6<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;
        const LFE: usize = 3;
        const SURROUND_L: usize = 4;
        const SURROUND_R: usize = 5;

        // Surround mix.
        let front_l = self.0[FRONT_L];
        let front_r = self.0[FRONT_R];
        let surround_l = self.0[SURROUND_L];
        let surround_r = self.0[SURROUND_R];
        let front = self.0[FRONT];
        let lfe = self.0[LFE];
        // Amplitude reduction.
        let amplitude = (N as f32 / 5.0).min(1.0);
        let mut frame = Frame::<S, N>::default()
            .pan(front_l * amplitude.into(), -30.0 / 360.0)
            .pan(front_r * amplitude.into(), 30.0 / 360.0)
            .pan(surround_l * amplitude.into(), -110.0 / 360.0)
            .pan(surround_r * amplitude.into(), 110.0 / 360.0)
            .pan(front * amplitude.into(), 0.0);
        // If no LFE channel, pan back center.
        if N < 5 {
            frame.pan(lfe * amplitude.into(), 0.5)
        } else {
            frame.0[3] = (lfe * amplitude.into()).into();
            frame
        }
    }

    #[inline(always)]
    fn to_7<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;
        const LFE: usize = 3;
        const BACK: usize = 4;
        const LEFT: usize = 5;
        const RIGHT: usize = 6;

        // Surround mix.
        let front_l = self.0[FRONT_L];
        let front_r = self.0[FRONT_R];
        let left = self.0[LEFT];
        let right = self.0[RIGHT];
        let front = self.0[FRONT];
        let lfe = self.0[LFE];
        let back = self.0[BACK];
        // Amplitude reduction.
        let amplitude = (N as f32 / 6.0).min(1.0);
        let mut frame = Frame::<S, N>::default()
            .pan(front_l * amplitude.into(), -30.0 / 360.0)
            .pan(front_r * amplitude.into(), 30.0 / 360.0)
            .pan(left * amplitude.into(), -90.0 / 360.0)
            .pan(right * amplitude.into(), 90.0 / 360.0)
            .pan(front * amplitude.into(), 0.0)
            .pan(back * amplitude.into(), 0.5);
        // If no LFE channel, pan back center.
        if N < 5 {
            frame.pan(lfe * amplitude.into(), 0.5)
        } else {
            frame.0[3] = (lfe * amplitude.into()).into();
            frame
        }
    }

    #[inline(always)]
    fn to_8<S: Channel + From<C>, const N: usize>(self) -> Frame<S, N> {
        const FRONT_L: usize = 0;
        const FRONT_R: usize = 1;
        const FRONT: usize = 2;
        const LFE: usize = 3;
        const BACK_L: usize = 4;
        const BACK_R: usize = 5;
        const LEFT: usize = 6;
        const RIGHT: usize = 7;

        // Surround mix.
        let front_l = self.0[FRONT_L];
        let front_r = self.0[FRONT_R];
        let left = self.0[LEFT];
        let right = self.0[RIGHT];
        let front = self.0[FRONT];
        let lfe = self.0[LFE];
        let back_l = self.0[BACK_L];
        let back_r = self.0[BACK_R];
        // Amplitude reduction.
        let amplitude = (N as f32 / 7.0).min(1.0);
        let mut frame = Frame::<S, N>::default()
            .pan(front_l * amplitude.into(), -30.0 / 360.0)
            .pan(front_r * amplitude.into(), 30.0 / 360.0)
            .pan(left * amplitude.into(), -90.0 / 360.0)
            .pan(right * amplitude.into(), 90.0 / 360.0)
            .pan(front * amplitude.into(), 0.0)
            .pan(back_l * amplitude.into(), -150.0 / 360.0)
            .pan(back_r * amplitude.into(), 150.0 / 360.0);
        // If no LFE channel, pan back center.
        if N < 5 {
            frame.pan(lfe * amplitude.into(), 0.5)
        } else {
            frame.0[3] = (lfe * amplitude.into()).into();
            frame
        }
    }
}

impl<C: Channel> Frame<C, 1> {
    /// Create a new mono interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(mono: C) -> Self {
        Self([mono])
    }
}

impl<C: Channel> Frame<C, 2> {
    /// Create a new stereo interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(left: C, right: C) -> Self {
        Self([left, right])
    }
}

impl<C: Channel> Frame<C, 3> {
    /// Create a new surround 3.0 interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(left: C, right: C, center: C) -> Self {
        Self([left, right, center])
    }
}

impl<C: Channel> Frame<C, 4> {
    /// Create a new surround 4.0 interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(left: C, right: C, back_left: C, back_right: C) -> Self {
        Self([left, right, back_left, back_right])
    }
}

impl<C: Channel> Frame<C, 5> {
    /// Create a new surround 5.0 interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(
        left: C,
        right: C,
        center: C,
        back_left: C,
        back_right: C,
    ) -> Self {
        Self([left, right, center, back_left, back_right])
    }
}

impl<C: Channel> Frame<C, 6> {
    /// Create a new surround 5.1 interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(
        left: C,
        right: C,
        center: C,
        lfe: C,
        back_left: C,
        back_right: C,
    ) -> Self {
        Self([left, right, center, lfe, back_left, back_right])
    }
}

impl<C: Channel> Frame<C, 7> {
    /// Create a new surround 6.1 interleaved audio frame from sample(s).
    #[inline(always)]
    pub fn new(
        left: C,
        right: C,
        center: C,
        lfe: C,
        back: C,
        side_left: C,
        side_right: C,
    ) -> Self {
        Self([left, right, center, lfe, back, side_left, side_right])
    }
}

impl<C: Channel> Frame<C, 8> {
    /// Create a new surround 7.1 interleaved audio frame from sample(s).
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        left: C,
        right: C,
        center: C,
        lfe: C,
        back_left: C,
        back_right: C,
        side_left: C,
        side_right: C,
    ) -> Self {
        Self([
            left, right, center, lfe, back_left, back_right, side_left,
            side_right,
        ])
    }
}

impl<C: Channel, const COUNT: usize> From<f32> for Frame<C, COUNT> {
    fn from(rhs: f32) -> Self {
        Frame([C::from(rhs); COUNT])
    }
}

impl<C: Channel, const COUNT: usize> Add for Frame<C, COUNT> {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a += *b;
        }
        self
    }
}

impl<C: Channel, const COUNT: usize> Sub for Frame<C, COUNT> {
    type Output = Self;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a -= *b;
        }
        self
    }
}

impl<C: Channel, const COUNT: usize> Mul for Frame<C, COUNT> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a *= *b;
        }
        self
    }
}

impl<C: Channel, const COUNT: usize> Neg for Frame<C, COUNT> {
    type Output = Self;

    #[inline(always)]
    fn neg(mut self) -> Self {
        for samp in self.0.iter_mut() {
            *samp = -*samp;
        }
        self
    }
}
