// Copyright © 2020-2021 The Fon Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::chan::Channel;
use crate::Frame;

/// Audio sink - a type that consumes audio samples.
pub trait Sink<Chan: Channel, const CH: usize> {
    /// Get the sample rate of the sink in hertz.
    fn sample_rate(&self) -> u32;

    /// Get the length of the sink in frames.
    ///
    /// Sinks must always have finite length.
    fn len(&self) -> usize;

    /// Sink audio samples from a frame iterator.
    ///
    /// **Warning**: if used incorrectly, this method may introduce audio
    /// aliasing.  To avoid that, make sure the sample rate of the frames from
    /// the iterator matches exactly the sample rate of the sink.
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<Chan, CH>>);

    /// Check if the sink is empty (length of zero).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
