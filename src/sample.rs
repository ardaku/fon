// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sample types

use crate::{chan::Channel, ops::Blend, private::Sealed};
use std::{fmt::Debug, mem::size_of};

const FRONT_LEFT: usize = 0; // CENTER for MONO, SIDE_LEFT for STEREO
const FRONT_RIGHT: usize = 1; // SIDE_RIGHT for STEREO
const BACK_LEFT: usize = 2;
const BACK_RIGHT: usize = 3;
const FRONT_CENTER: usize = 4;
const LFE: usize = 5;
const SIDE_LEFT: usize = 6;
const SIDE_RIGHT: usize = 7;

/// Sample with one [channel](chan/trait.Channel.html).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Sample1<C: Channel> {
    channels: [C; 1],
}

impl<C: Channel> Sample1<C> {
    /// Create a one-channel Sample.
    pub fn new<H>(one: H) -> Self
    where
        C: From<H>,
    {
        let one = C::from(one);
        let channels = [one];
        Self { channels }
    }
}

impl<C: Channel> Sample for Sample1<C> {
    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        let one = ch[0];
        Self::new::<C>(one)
    }
}

/// Sample with two [channel](chan/trait.Channel.html)s.
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Sample2<C: Channel> {
    channels: [C; 2],
}

impl<C: Channel> Sample2<C> {
    /// Create a two-channel Sample.
    pub fn new<H>(one: H, two: H) -> Self
    where
        C: From<H>,
    {
        let one = C::from(one);
        let two = C::from(two);
        let channels = [one, two];
        Self { channels }
    }
}

impl<C: Channel> Sample for Sample2<C> {
    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        let one = ch[0];
        let two = ch[1];
        Self::new::<C>(one, two)
    }
}

/// Sample with four [channel](chan/trait.Channel.html)s.
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Sample4<C: Channel> {
    channels: [C; 4],
}

impl<C: Channel> Sample4<C> {
    /// Create a four-channel Sample.
    pub fn new<H>(one: H, two: H, three: H, four: H) -> Self
    where
        C: From<H>,
    {
        let one = C::from(one);
        let two = C::from(two);
        let three = C::from(three);
        let four = C::from(four);
        let channels = [one, two, three, four];
        Self { channels }
    }
}

impl<C: Channel> Sample for Sample4<C> {
    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        let one = ch[0];
        let two = ch[1];
        let three = ch[2];
        let four = ch[3];
        Self::new::<C>(one, two, three, four)
    }
}

/// Sample with six [channel](chan/trait.Channel.html)s.
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Sample6<C: Channel> {
    channels: [C; 6],
}

impl<C: Channel> Sample6<C> {
    /// Create a six-channel Sample.
    pub fn new<H>(one: H, two: H, three: H, four: H, five: H, six: H) -> Self
    where
        C: From<H>,
    {
        let one = C::from(one);
        let two = C::from(two);
        let three = C::from(three);
        let four = C::from(four);
        let five = C::from(five);
        let six = C::from(six);
        let channels = [one, two, three, four, five, six];
        Self { channels }
    }
}

impl<C: Channel> Sample for Sample6<C> {
    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        let one = ch[0];
        let two = ch[1];
        let three = ch[2];
        let four = ch[3];
        let five = ch[4];
        let six = ch[5];
        Self::new::<C>(one, two, three, four, five, six)
    }
}

/// Sample with six [channel](chan/trait.Channel.html)s.
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Sample8<C: Channel> {
    channels: [C; 8],
}

impl<C: Channel> Sample8<C> {
    /// Create an eight-channel Sample.
    #[allow(clippy::too_many_arguments)]
    pub fn new<H>(one: H, two: H, three: H, four: H, five: H, six: H, seven: H, eight: H) -> Self
    where
        C: From<H>,
    {
        let one = C::from(one);
        let two = C::from(two);
        let three = C::from(three);
        let four = C::from(four);
        let five = C::from(five);
        let six = C::from(six);
        let seven = C::from(seven);
        let eight = C::from(eight);
        let channels = [one, two, three, four, five, six, seven, eight];
        Self { channels }
    }
}

impl<C: Channel> Sample for Sample8<C> {
    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        let one = ch[0];
        let two = ch[1];
        let three = ch[2];
        let four = ch[3];
        let five = ch[4];
        let six = ch[5];
        let seven = ch[6];
        let eight = ch[7];
        Self::new::<C>(one, two, three, four, five, six, seven, eight)
    }
}

/// Sample - A number of [channel]s.
///
/// [channel]: ../chan/trait.Channel.html
pub trait Sample: Clone + Copy + Debug + Default + PartialEq + Sealed {
    /// Channel type
    type Chan: Channel;

    /// Number of channels
    const CHAN_COUNT: usize = size_of::<Self>() / size_of::<Self::Chan>();

    /// Get the channels.
    fn channels(&self) -> &[Self::Chan];

    /// Get the channels mutably.
    fn channels_mut(&mut self) -> &mut [Self::Chan];

    /// Make a pixel from a slice of channels.
    fn from_channels(ch: &[Self::Chan]) -> Self;

    /// Linear interpolation.
    fn lerp(&self, rhs: Self, t: Self) -> Self {
        let mut out = Self::default();
        let main = out.channels_mut().iter_mut().zip(self.channels().iter());
        let other = rhs.channels().iter().zip(t.channels().iter());
        for ((out, this), (rhs, t)) in main.zip(other) {
            *out = this.lerp(*rhs, *t);
        }
        out
    }

    /// Synthesis of a sample with a slice of samples.
    fn blend_sample<O>(dst: &mut [Self], sample: &Self, op: O)
    where
        O: Blend,
    {
        for d in dst.iter_mut() {
            d.blend(sample, op);
        }
    }

    /// Synthesis of two slices of samples.
    fn blend_slice<O>(dst: &mut [Self], src: &[Self], op: O)
    where
        O: Blend,
    {
        for (d, s) in dst.iter_mut().zip(src) {
            d.blend(s, op);
        }
    }

    /// Synthesize two samples together.
    fn blend<O>(&mut self, src: &Self, _op: O)
    where
        O: Blend,
    {
        for (d, s) in self.channels_mut().iter_mut().zip(src.channels().iter()) {
            O::synthesize(d, s)
        }
    }

    /// Convert a sample to another format.
    #[inline(always)]
    fn convert<D>(self) -> D
    where
        D: Sample,
        D::Chan: From<Self::Chan> + From<f64>,
    {
        // Convert channels
        match (Self::CHAN_COUNT, D::CHAN_COUNT) {
            // 1:1 sampling (no resample)
            (a, b) if a == b => {
                let mut chans = [D::Chan::MID; 8];
                for (d, s) in chans.iter_mut().zip(self.channels().iter()) {
                    *d = (*s).into();
                }
                D::from_channels(&chans[..self.channels().len()])
            }
            // Downsampling
            (2, 1) => {
                let mut sum = 0.0;
                for chan in self.channels() {
                    sum += chan.to_f64() * 0.5;
                }
                D::from_channels(&[D::Chan::from(sum)])
            }
            (6, 1) => {
                let mut sum = 0.0;
                sum += self.channels()[FRONT_LEFT].to_f64() * (1.0 / 5.0);
                sum += self.channels()[FRONT_RIGHT].to_f64() * (1.0 / 5.0);
                sum += self.channels()[BACK_LEFT].to_f64() * (1.0 / 5.0);
                sum += self.channels()[BACK_RIGHT].to_f64() * (1.0 / 5.0);
                sum += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 5.0);
                sum += self.channels()[LFE].to_f64();
                D::from_channels(&[D::Chan::from(sum)])
            }
            (8, 1) => {
                let mut sum = 0.0;
                sum += self.channels()[FRONT_LEFT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[FRONT_RIGHT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[BACK_LEFT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[BACK_RIGHT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 7.0);
                sum += self.channels()[SIDE_LEFT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[SIDE_RIGHT].to_f64() * (1.0 / 7.0);
                sum += self.channels()[LFE].to_f64();
                D::from_channels(&[D::Chan::from(sum)])
            }
            (6, 2) => {
                let mut left = 0.0;
                let mut right = 0.0;

                left += self.channels()[FRONT_LEFT].to_f64() * (1.0 / 3.0);
                right += self.channels()[FRONT_RIGHT].to_f64() * (1.0 / 3.0);
                left += self.channels()[BACK_LEFT].to_f64() * (1.0 / 3.0);
                right += self.channels()[BACK_RIGHT].to_f64() * (1.0 / 3.0);
                left += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 3.0);
                right += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 3.0);
                left += self.channels()[LFE].to_f64();
                right += self.channels()[LFE].to_f64();

                D::from_channels(&[D::Chan::from(left), D::Chan::from(right)])
            }
            (8, 2) => {
                let mut left = 0.0;
                let mut right = 0.0;

                left += self.channels()[FRONT_LEFT].to_f64() * (1.0 / 4.0);
                right += self.channels()[FRONT_RIGHT].to_f64() * (1.0 / 4.0);
                left += self.channels()[BACK_LEFT].to_f64() * (1.0 / 4.0);
                right += self.channels()[BACK_RIGHT].to_f64() * (1.0 / 4.0);
                left += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 4.0);
                right += self.channels()[FRONT_CENTER].to_f64() * (1.0 / 4.0);
                left += self.channels()[SIDE_LEFT].to_f64() * (1.0 / 4.0);
                right += self.channels()[SIDE_RIGHT].to_f64() * (1.0 / 4.0);
                left += self.channels()[LFE].to_f64();
                right += self.channels()[LFE].to_f64();

                D::from_channels(&[D::Chan::from(left), D::Chan::from(right)])
            }
            (8, 6) => {
                let mut left = self.channels()[FRONT_LEFT].to_f64() * (2.0 / 3.0);
                let mut right = self.channels()[FRONT_RIGHT].to_f64() * (2.0 / 3.0);
                let center = self.channels()[FRONT_CENTER].to_f64();
                let mut back_left = self.channels()[BACK_LEFT].to_f64() * (2.0 / 3.0);
                let mut back_right = self.channels()[BACK_RIGHT].to_f64() * (2.0 / 3.0);
                let lfe = self.channels()[LFE].to_f64();
                left += self.channels()[SIDE_LEFT].to_f64() * (1.0 / 3.0);
                right += self.channels()[SIDE_RIGHT].to_f64() * (1.0 / 3.0);
                back_left += self.channels()[SIDE_LEFT].to_f64() * (1.0 / 3.0);
                back_right += self.channels()[SIDE_RIGHT].to_f64() * (1.0 / 3.0);

                D::from_channels(&[
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(back_left),
                    D::Chan::from(back_right),
                    D::Chan::from(center),
                    D::Chan::from(lfe),
                ])
            }
            // Upsampling
            (1, 2) => {
                let mono = self.channels()[0];
                let channels = [D::Chan::from(mono), D::Chan::from(mono)];
                D::from_channels(&channels)
            }
            (1, 6) => {
                let mono = self.channels()[0];
                D::from_channels(&[
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                ])
            }
            (1, 8) => {
                let mono = self.channels()[0];
                D::from_channels(&[
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                    D::Chan::from(mono),
                ])
            }
            (2, 6) => {
                let left = self.channels()[FRONT_LEFT].to_f64();
                let right = self.channels()[FRONT_RIGHT].to_f64();
                let center = left * 0.5 + right * 0.5;
                let lfe = D::Chan::MID.to_f64();
                D::from_channels(&[
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(center),
                    D::Chan::from(lfe),
                ])
            }
            (2, 8) => {
                let left = self.channels()[FRONT_LEFT].to_f64();
                let right = self.channels()[FRONT_RIGHT].to_f64();
                let center = left * 0.5 + right * 0.5;
                let lfe = D::Chan::MID.to_f64();
                D::from_channels(&[
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(center),
                    D::Chan::from(lfe),
                    D::Chan::from(left),
                    D::Chan::from(right),
                ])
            }
            (5, 8) => {
                let left = self.channels()[FRONT_LEFT].to_f64();
                let right = self.channels()[FRONT_RIGHT].to_f64();
                let center = self.channels()[FRONT_CENTER].to_f64();
                let back_left = self.channels()[BACK_LEFT].to_f64();
                let back_right = self.channels()[BACK_RIGHT].to_f64();
                let lfe = self.channels()[LFE].to_f64();
                let side_left = (left + back_left) * 0.5;
                let side_right = (right + back_right) * 0.5;
                D::from_channels(&[
                    D::Chan::from(left),
                    D::Chan::from(right),
                    D::Chan::from(back_left),
                    D::Chan::from(back_right),
                    D::Chan::from(center),
                    D::Chan::from(lfe),
                    D::Chan::from(side_left),
                    D::Chan::from(side_right),
                ])
            }
            // Unreachable because of sealed traits
            (_, _) => unreachable!(),
        }
    }
}
