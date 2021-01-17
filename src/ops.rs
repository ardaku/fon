// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Audio mixing operations.

use crate::{chan::Channel, Frame};
use core::any::Any;

/// Blending operation for mixing
pub trait Blend: Any + Copy + Clone {
    /// blend a destination and source.
    fn mix<C: Channel>(dst: &mut C, src: &C);

    /// Compose frames.
    #[inline(always)]
    fn mix_frames<F: Frame>(dst: F, src: F) -> F {
        let mut dst = dst;
        for (dst, src) in
            dst.channels_mut().iter_mut().zip(src.channels().iter())
        {
            Self::mix(dst, src);
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
/// Standard audio mixing.  Addition of signals (source added to destination).
#[derive(Clone, Copy, Debug)]
pub struct Plus;
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
    fn mix<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src;
    }
}

impl Blend for Dest {
    #[inline(always)]
    fn mix<C: Channel>(_dst: &mut C, _src: &C) {
        // leave _dst as is
    }
}

impl Blend for Clear {
    #[inline(always)]
    fn mix<C: Channel>(dst: &mut C, _src: &C) {
        *dst = C::default();
    }
}

impl Blend for Amplify {
    #[inline(always)]
    fn mix<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src * *dst;
    }
}

impl Blend for Plus {
    #[inline(always)]
    fn mix<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src + *dst;
    }
}

impl Blend for Compress {
    #[inline(always)]
    fn mix<C: Channel>(_dst: &mut C, _src: &C) {
        todo!()
    }
}

impl Blend for Pan {
    #[inline(always)]
    fn mix<C: Channel>(_dst: &mut C, _src: &C) {
        panic!("Panning is useless on one channel!")
    }

    #[inline(always)]
    fn mix_frames<F: Frame>(dst: F, src: F) -> F {
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
