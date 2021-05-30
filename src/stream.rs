// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::chan::Channel;
use crate::frame::Frame;
use crate::ops::Ops;
use crate::Audio;

/// Audio stream - a type that generates audio samples.
pub trait Stream<Chan, const CH: usize, const SR: u32>
where
    Chan: Channel,
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Stream audio, appending `len` samples to the end of `buffer`.
    fn extend<C: Channel>(&mut self, buffer: &mut Audio<C, CH, SR>, len: usize)
    where
        C: From<Chan>,
        Frame<C, CH>: Ops<C>;

    /// Stream audio into `buffer`, overwriting the samples.
    #[inline(always)]
    fn stream<C: Channel>(&mut self, buffer: &mut Audio<C, CH, SR>)
    where
        C: From<Chan>,
        Frame<C, CH>: Ops<C>,
    {
        // Get old (original) length.
        let len = buffer.len();
        // Empty buffer.
        buffer.clear();
        // Extend back to old (original) length.
        self.extend(buffer, len);
    }
}
