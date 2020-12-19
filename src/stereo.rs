// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Stereo speaker configuration and types.

use crate::{
    chan::{Ch16, Ch32, Ch64, Ch8, Channel},
    sample::Sample,
};

/// Stereo sample format (left channel, right channel).
#[derive(Default, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Stereo<C: Channel> {
    channels: [C; 2],
}

impl<C: Channel> Stereo<C> {
    /// Create a two-channel Sample.
    pub fn new<H>(one: H, two: H) -> Self
    where
        C: From<H>,
    {
        let channels = [C::from(one), C::from(two)];
        Self { channels }
    }
}

impl<C: Channel> Sample for Stereo<C> {
    const CONFIG: &'static [[f64; 2]] = &[[0.0, 0.5], [0.5, 1.0]];

    type Chan = C;

    fn channels(&self) -> &[Self::Chan] {
        &self.channels
    }

    fn channels_mut(&mut self) -> &mut [Self::Chan] {
        &mut self.channels
    }

    fn from_channels(ch: &[Self::Chan]) -> Self {
        Self::new::<C>(ch[0], ch[1])
    }
}

/// Stereo [8-bit PCM](../chan/struct.Ch8.html) format.
pub type Stereo8 = Stereo<Ch8>;
/// Stereo [16-bit PCM](../chan/struct.Ch16.html) format.
pub type Stereo16 = Stereo<Ch16>;
/// Stereo [32-bit Floating Point](../chan/struct.Ch32.html) format.
pub type Stereo32 = Stereo<Ch32>;
/// Stereo [64-bit Floating Point](../chan/struct.Ch64.html) format.
pub type Stereo64 = Stereo<Ch64>;
