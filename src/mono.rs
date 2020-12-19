// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mono speaker configuration and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8, Channel},
    sample::Sample,
};

/// Mono sample format (one channel).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Mono<C: Channel> {
    channels: [C; 1],
}

impl<C: Channel> Mono<C> {
    /// Create a one-channel Sample.
    pub fn new<H>(one: H) -> Self
    where
        C: From<H>,
    {
        let channels = [C::from(one)];
        Self { channels }
    }
}

impl<C: Channel> Sample for Mono<C> {
    const CONFIG: &'static [[f64; 2]] = &[[0.0, 1.0]];

    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        Self::new::<C>(ch[0])
    }
}

/// Mono [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Mono8 = Mono<Ch8>;
/// Mono [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Mono16 = Mono<Ch16>;
/// Mono [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Mono32 = Mono<Ch32>;
/// Mono [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Mono64 = Mono<Ch64>;
