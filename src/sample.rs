// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Sample types

use crate::{chan::Channel, ops::Blend};
use std::{fmt::Debug, mem::size_of};

/// Sample - A number of [channel]s.
///
/// [channel]: ../chan/trait.Channel.html
pub trait Sample: Clone + Copy + Debug + Default + PartialEq {
    /// Channel type
    type Chan: Channel;

    /// Number of channels
    const CHAN_COUNT: usize = size_of::<Self>() / size_of::<Self::Chan>();

    /// Speaker configuration (Stored as a list of relative areas, stored as
    /// cycles - 0.25 is left, 0.5 is behind, 0.75 is right, 1.0 is front).
    /// These must be listed in increasing order counterclockwise/widdershins.
    const CONFIG: &'static [[f64; 2]];

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
        // FIXME: Remove allocation from this function.
        let mut output = vec![0.0f64; D::CONFIG.len()];
        let mut config = Self::CONFIG.iter().enumerate().peekable();

        for (i, out) in output.iter_mut().enumerate() {
            let [out_start, out_end] = D::CONFIG[i];
            let one = out_end - out_start;

            // Calculate amount of each input to go in the output.
            'amt: while let Some((j, [start, end])) = config.peek() {
                if *end > out_end {
                    let cover = 1.0 - (start / one);
                    *out += cover * self.channels()[*j].to_f64();
                    // Should be reused if falls after, end of arc.
                    break 'amt;
                }
                let cover = (end - start) / one;
                *out += cover * self.channels()[*j].to_f64();
                // unwrap: never fails because of previous peek.
                config.next().unwrap();
            }

            // Clamp to preserve type invariant
            *out = out.min(1.0).max(0.0);
        }

        let mut out = vec![];

        for i in output {
            out.push(D::Chan::from(i));
        }

        D::from_channels(&out[..])
    }
}
