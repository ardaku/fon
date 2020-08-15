// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Synthesis and mixing operations.
//!
//! Used in `Audio` methods `blend_sample` and `blend_audio`.

use crate::chan::Channel;
use core::any::Any;

/// Blending operation for mixing
pub trait Blend: Any + Copy + Clone {
    /// Synthesize to destination by blending destination and source.
    fn synthesize<C: Channel>(dst: &mut C, src: &C);
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

impl Blend for Src {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src;
    }
}

impl Blend for Dest {
    fn synthesize<C: Channel>(_dst: &mut C, _src: &C) {
        // leave _dst as is
    }
}

impl Blend for Clear {
    fn synthesize<C: Channel>(dst: &mut C, _src: &C) {
        *dst = C::default();
    }
}

impl Blend for Amplify {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src * *dst;
    }
}

impl Blend for Mix {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = *src + *dst;
    }
}

impl Blend for Compress {
    fn synthesize<C: Channel>(_dst: &mut C, _src: &C) {
        todo!()
    }
}



/// Minimum of destination and source
#[derive(Clone, Copy, Debug)]
pub struct Min;
/// Maximum of destination and source
#[derive(Clone, Copy, Debug)]
pub struct Max;
/// Raise destination to the power of source
#[derive(Clone, Copy, Debug)]
pub struct Pow;
/// Apply absolute value function to destination (useful for multiplying
/// waveforms together without octave jump), multiplied by source.
#[derive(Clone, Copy, Debug)]
pub struct Abs;
/// Hard clipping and amplification at source power to destination.
#[derive(Clone, Copy, Debug)]
pub struct ClipHard;
/// Soft clipping and amplification at source power to destination.
#[derive(Clone, Copy, Debug)]
pub struct ClipSoft;

impl Blend for Min {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = (*dst).min(*src);
    }
}

impl Blend for Max {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = (*dst).max(*src);
    }
}

impl Blend for Pow {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = C::from(dst.to_f64().powf(src.to_f64()));
    }
}

impl Blend for Abs {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = C::from(dst.to_f64().abs()) * *src;
    }
}

impl Blend for ClipHard {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        *dst = C::from((dst.to_f64() / src.to_f64()).min(1.0).max(-1.0));
    }
}

impl Blend for ClipSoft {
    fn synthesize<C: Channel>(dst: &mut C, src: &C) {
        let input = dst.to_f64();
        let volume = src.to_f64().recip();
        let max = (1.0 / (1.0 + (-volume).exp())) * 2.0 - 1.0;
        let out = ((1.0 / (1.0 + (input * -volume).exp())) * 2.0 - 1.0) / max;
        *dst = C::from(out);
    }
}
