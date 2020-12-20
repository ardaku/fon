// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Sample types

use crate::{
    chan::{Ch64, Channel},
    ops::Blend,
    mono::Mono,
};
use std::{fmt::Debug, mem::size_of};

/// Returns how much src covers dst.  Units are counterclockwise from 0 to 1+
fn arc_cover(dst: [f64; 2], mut src: [f64; 2]) -> f64 {
    // Check if a point lies within an arc.
    fn point_in_arc(arc: [f64; 2], pt: f64) -> bool {
        let dst = [arc[0] % 1.0, arc[1] % 1.0];
        let pt = pt % 1.0;
        if dst[0] > dst[1] { // Inverted range; pt must fall in dst[1] to dst[0]
            pt > dst[1] && pt < dst[0]
        } else { // Regular range; pt must fall in dst[0] to dst[1]
            pt > dst[0] && pt < dst[1]
        }
    }
    // Adjust area when src begins before dst (outside overlapping zone).
    if !point_in_arc(dst, src[0]) {
        src[0] = dst[0];
    }
    // Adjust area when src ends after dst (outside overlapping zone).
    if !point_in_arc(dst, src[1]) {
        src[1] = dst[1];
    }
    // Calculate areas
    let src_area = src[1] - src[0];
    let dst_area = dst[1] - dst[0];
    // Amount of dst cover by src
    src_area / dst_area
}

/// Sample - A number of [channel]s.
///
/// [channel]: ../chan/trait.Channel.html
pub trait Sample: Clone + Copy + Debug + Default + PartialEq + Unpin {
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

    /// Pan a channel into this Sample type, units are in clockwise rotations.
    fn from_channel_panned(ch: Self::Chan, cw_rot: f64) -> Self {
        let mut out = [Self::Chan::default(); 8];

        // Convert to widdershins rotations offset by a quarter clockwise (-ws).
        let ws_rot = 1.0 - (cw_rot + 0.25) % 1.0;

        // Cycle through configurations.
        for (i, dst) in Self::CONFIG.iter().enumerate() {
            out[i] = Self::Chan::from(ch.to_f64() * arc_cover(*dst, [ws_rot, ws_rot + 0.5]));
        }

        Self::from_channels(&out)
    }

    /// Pan a mono sample into this Sample type, units are in clockwise
    /// rotations.
    #[inline(always)]
    fn from_mono_panned(ch: Mono<Self::Chan>, cw_rot: f64) -> Self {
        Self::from_channel_panned(ch.channels()[0], cw_rot)
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

    /// Synthesize two samples together.
    #[inline(always)]
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
    fn convert<D: Sample>(self) -> D {
        // Limit to 8 channels.
        let output = &mut [0.0f64; 8][..D::CONFIG.len()];
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
        }

        let out = &mut [D::Chan::default(); 8][..];

        for (o, i) in out.iter_mut().zip(output.iter()) {
            *o = D::Chan::from(Ch64::from(*i));
        }

        D::from_channels(out)
    }
}

impl<T: Sample> crate::Stream<T> for T {
    fn stream<O: Blend, K: crate::Sink>(&mut self, sink: &mut K, op: O) {
        for _ in 0..sink.capacity() {
            sink.sink_sample(*self, op);
        }
    }

    fn sample_rate(&self) -> u32 {
        panic!("No sample rate for constant stream.")
    }

    fn stream_sample(&mut self) -> Option<T> {
        Some(*self)
    }

    fn resampler(&mut self) -> &mut crate::Resampler<T> {
        panic!("No resampler for constant stream.")
    }
}
