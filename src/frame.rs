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

use crate::{chan::Channel, mono::Mono, stereo::Stereo, surround51::Surround};
use core::{
    any::TypeId,
    fmt::Debug,
    mem::size_of,
    ops::{
        Add, Mul, Neg, Sub,
    },
};

/// Frame - A number of interleaved sample [channel]s.
///
/// [channel]: crate::chan::Channel
pub trait Frame:
    Clone
    + Copy
    + Debug
    + Default
    + PartialEq
    + Unpin
    + Add<Output = Self>
    + Mul<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
    + Iterator<Item = Self>
    + 'static
{
    /// Channel type
    type Chan: Channel;

    /// Number of channels
    const CHAN_COUNT: usize = size_of::<Self>() / size_of::<Self::Chan>();

    /// Speaker configuration (Stored as a list of locations, -1.0 refers to
    /// the back going left, 1.0 also refers to the back - but going right, and
    /// 0.0 refers to center (straight ahead).  These should be listed from
    /// left to right, does not include LFE.
    const CONFIG: &'static [f64];

    /// Get the channels.
    fn channels(&self) -> &[Self::Chan];

    /// Get the channels mutably.
    fn channels_mut(&mut self) -> &mut [Self::Chan];

    /// Make an audio frame with all channels set from a floating point value.
    fn from(value: f32) -> Self {
        let mut ret = Self::default();
        for chan in ret.channels_mut() {
            *chan = Self::Chan::from(value);
        }
        ret
    }

    /// Make an audio frame from a singular channel.
    fn from_channel(ch: Self::Chan) -> Self {
        let mut ret = Self::default();
        for chan in ret.channels_mut() {
            *chan = ch;
        }
        ret
    }

    /// Make an audio frame from a slice of channels.
    fn from_channels(ch: &[Self::Chan]) -> Self;

    /// Make an audio frame from a mono frame.
    fn from_mono(frame: Mono<Self::Chan>) -> Self {
        Self::from_channel(frame.channels()[0])
    }

    /// Linear interpolation.
    #[inline(always)]
    fn lerp(&self, rhs: Self, t: Self) -> Self {
        let mut out = Self::default();
        let main = out.channels_mut().iter_mut().zip(self.channels().iter());
        let other = rhs.channels().iter().zip(t.channels().iter());
        for ((out, this), (rhs, t)) in main.zip(other) {
            *out = this.lerp(*rhs, *t);
        }
        out
    }

    /// Convert a sample to another format.
    #[inline(always)]
    fn convert<D: Frame>(self) -> D {
        match (TypeId::of::<Self>(), TypeId::of::<D>()) {
            (a, b)
                if (a == TypeId::of::<Mono<Self::Chan>>()
                    && b == TypeId::of::<Mono<D::Chan>>())
                    || (a == TypeId::of::<Stereo<Self::Chan>>()
                        && b == TypeId::of::<Stereo<D::Chan>>())
                    || (a == TypeId::of::<Surround<Self::Chan>>()
                        && b == TypeId::of::<Surround<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 6];
                // Same type, 1:1
                for (src, dst) in self.channels().iter().zip(out.iter_mut()) {
                    *dst = D::Chan::from(src.to_f32());
                }
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Mono<Self::Chan>>()
                    && b == TypeId::of::<Stereo<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 2];
                // Mono -> Stereo, Duplicate The Channel
                out[0] = D::Chan::from(self.channels()[0].to_f32());
                out[1] = D::Chan::from(self.channels()[0].to_f32());
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Mono<Self::Chan>>()
                    && b == TypeId::of::<Surround<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 6];
                // Mono -> Surround (Mono -> Stereo -> Surround)
                out[1] = D::Chan::from(self.channels()[0].to_f32());
                out[3] = D::Chan::from(self.channels()[0].to_f32());
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Stereo<Self::Chan>>()
                    && b == TypeId::of::<Surround<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 6];
                // Stereo -> Surround
                out[1] = D::Chan::from(self.channels()[0].to_f32());
                out[3] = D::Chan::from(self.channels()[1].to_f32());
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Surround<Self::Chan>>()
                    && b == TypeId::of::<Stereo<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 2];
                // Surround -> Stereo
                out[0] = D::Chan::from(self.channels()[1].to_f32());
                out[1] = D::Chan::from(self.channels()[3].to_f32());
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Surround<Self::Chan>>()
                    && b == TypeId::of::<Mono<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 1];
                // Surround -> Stereo -> Mono
                out[0] = D::Chan::from(
                    (self.channels()[1].to_f32() + self.channels()[3].to_f32())
                        * 0.5,
                );
                //
                D::from_channels(&out)
            }
            (a, b)
                if (a == TypeId::of::<Stereo<Self::Chan>>()
                    && b == TypeId::of::<Mono<D::Chan>>()) =>
            {
                let mut out = [D::Chan::MID; 1];
                // Stereo -> Mono
                out[0] = D::Chan::from(
                    (self.channels()[0].to_f32() + self.channels()[1].to_f32())
                        * 0.5,
                );
                //
                D::from_channels(&out)
            }
            _ => panic!(
                "Cannot convert custom speaker configurations, \
                implement custom Frame::convert() method to override."
            ),
        }
    }
}

impl<T: Frame> crate::Stream<T> for T {
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
