// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Audio mixing operations.

use crate::{chan::Channel, Frame};
use core::any::Any;

/// Blending operation for mixing
pub trait Blend: Any + Copy + Clone {
    /// blend a destination and source.
    fn blend<C: Channel>(dst: &mut C, src: &C);

    /// Compose frames.
    #[inline(always)]
    fn blend_frames<F: Frame>(dst: F, src: F) -> F {
        let mut dst = dst;
        for (dst, src) in
            dst.channels_mut().iter_mut().zip(src.channels().iter())
        {
            Self::blend(dst, src);
        }
        dst
    }
}

/// Source only (ignore destination)
#[derive(Clone, Copy, Debug)]
pub struct Src;
/// Destination only (ignore source)
#[derive(Clone, Copy, Debug)]
pub struct Dest;
/// Clear (set to default)
#[derive(Clone, Copy, Debug)]
pub struct Clear;
/// Amplify one sample by another
#[derive(Clone, Copy, Debug)]
pub struct Amplify;
/// Standard audio mixing.  Addition of signals
#[derive(Clone, Copy, Debug)]
pub struct Mix;
/// Compression of channel (based on another channel)
#[derive(Clone, Copy, Debug)]
pub struct Compress;
/// Pan channels
///
/// (MIN is left pan to back, MAX is right pan to back, MID is straight ahead).
/// Each channel is panned separately to the destination.  The source panning is
/// ignored.
#[derive(Clone, Copy, Debug)]
pub struct Pan;

impl Blend for Src {
    #[inline(always)]
    fn blend<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src;
    }
}

impl Blend for Dest {
    #[inline(always)]
    fn blend<C: Channel>(_dst: &mut C, _src: &C) {
        // leave _dst as is
    }
}

impl Blend for Clear {
    #[inline(always)]
    fn blend<C: Channel>(dst: &mut C, _src: &C) {
        *dst = C::default();
    }
}

impl Blend for Amplify {
    #[inline(always)]
    fn blend<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src * *dst;
    }
}

impl Blend for Mix {
    #[inline(always)]
    fn blend<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src + *dst;
    }
}

impl Blend for Compress {
    #[inline(always)]
    fn blend<C: Channel>(_dst: &mut C, _src: &C) {
        todo!()
    }
}

impl Blend for Pan {
    #[inline(always)]
    fn blend<C: Channel>(_dst: &mut C, _src: &C) {
        panic!("Panning is useless on one channel!")
    }

    #[inline(always)]
    fn blend_frames<F: Frame>(dst: F, src: F) -> F {
        let mut out = F::default();
        for (d, s) in dst.channels().iter().zip(src.channels().iter()) {
            // Get the panning amount for this channel.
            let s = s.to_f64();
            // Figure out which two destination channels the audio applies to.
            let mut start = F::CONFIG.len() - 1;
            for (i, location) in F::CONFIG.iter().enumerate() {
                if s >= *location {
                    start = i;
                    break;
                }
            }
            let end = (start + 1) % F::CONFIG.len();
            // Get distance between channels
            let mut dist = F::CONFIG[end] - F::CONFIG[start];
            if dist < 0.0 {
                dist = 2.0 - dist;
            }
            // Get closeness between the two channels (0 thru 1)
            let closeness = (s - F::CONFIG[end].min(F::CONFIG[start])) / dist;
            // Constant Power Panning
            let angle = closeness * core::f64::consts::FRAC_PI_2;
            out.channels_mut()[start] += *d * F::Chan::from_f64(angle.cos());
            out.channels_mut()[end] += *d * F::Chan::from_f64(angle.sin());
        }
        out
    }
}
