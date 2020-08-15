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
