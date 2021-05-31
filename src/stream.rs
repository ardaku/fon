// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::chan::{Ch32, Channel};
use crate::resampler::Resampler;
use crate::Audio;

/// Audio stream - a type that generates audio samples.
pub trait Stream<Chan, const CH: usize>: Sized
where
    Chan: Channel,
{
    /// Get the sample rate of the stream in hertz.
    fn sample_rate(&self) -> Option<u32>;

    /// Stream audio, appending `len` samples to the end of `buffer`.
    ///
    /// This method should always add `len` samples; If there are not enough
    /// then this should append zero samples at the end.
    fn extend<C: Channel, const N: usize>(
        &mut self,
        buffer: &mut Audio<C, N>,
        len: usize,
    ) where
        C: From<Chan>;

    /// Stream audio into `buffer`, overwriting the samples.
    #[inline(always)]
    fn stream<C: Channel, const N: usize>(&mut self, buffer: &mut Audio<C, N>)
    where
        C: From<Chan>,
    {
        // Get old (original) length.
        let len = buffer.len();
        // Empty buffer.
        buffer.clear();
        // Extend back to old (original) length.
        self.extend(buffer, len);
    }

    /// Convert this stream to a [`Stream`](crate::Stream) of a different
    /// sample rate.
    #[inline(always)]
    fn resample(self, sample_rate_hz: u32) -> Resampler<Self, Chan, CH>
    where
        Ch32: From<Chan>,
    {
        crate::resampler::Resampler::new(sample_rate_hz, self)
    }
}
