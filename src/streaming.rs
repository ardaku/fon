// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::{math, Frame};

/// Context for an audio resampler.
#[derive(Default, Debug, Copy, Clone)]
pub struct Resampler<F: Frame> {
    /// Left over partial frame.
    partial: F,
    /// Left over partial index.
    offseti: f64,
}

impl<F: Frame> Resampler<F> {
    /// Create a new resampler context.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Audio sink - a type that consumes audio samples.
pub trait Sink<F: Frame>: Sized {
    /// Get the (target) sample rate of the [`Sink`](crate::Sink).
    fn sample_rate(&self) -> f64;

    /// Get the [`Resampler`](crate::Resampler) context for this
    /// [`Sink`](crate::Sink).
    fn resampler(&mut self) -> &mut Resampler<F>;

    /// Get the (target) audio [Frame](crate::Frame) buffer of the
    /// [`Sink`](crate::Sink).
    fn buffer(&mut self) -> &mut [F];

    /// [`Stream`](crate::Stream) audio into this audio [`Sink`](crate::Sink).
    #[inline(always)]
    fn sink<S: Frame, M: Stream<S>>(&mut self, mut stream: M) {
        // Ratio of destination samples per stream samples.
        let ratio = if let Some(stream_sr) = stream.sample_rate() {
            self.sample_rate() / stream_sr
        } else {
            stream.set_sample_rate(self.sample_rate());
            1.0
        };

        // Add left over audio.
        let partial = self.resampler().partial;
        if let Some(dst) = self.buffer().get_mut(0) {
            *dst = partial;
        }

        // Go through each source sample and add to destination.
        let mut srclen = stream.len();
        for (i, src) in stream.into_iter().enumerate() {
            // Calculate destination index.
            let j = ratio * i as f64 + self.resampler().offseti;
            let ceil = math::ceil_usize(j);
            let floor = j as usize;
            let ceil_f64 = (j % 1.0).min(ratio);
            let ceil_a = F::from_f64(ceil_f64);
            let floor_a = F::from_f64(ratio - ceil_f64);
            let src: F = src.convert();
            if let Some(buf) = self.buffer().get_mut(floor) {
                *buf += src * floor_a;
            } else {
                srclen = Some(i);
                break;
            }
            if let Some(buf) = self.buffer().get_mut(ceil) {
                *buf += src * ceil_a;
            } else {
                self.resampler().partial += src * ceil_a;
            }
        }

        // Set offseti
        self.resampler().offseti = (ratio * (srclen.unwrap() - 1) as f64
            + self.resampler().offseti)
            % 1.0;
    }
}

/// Audio stream - a type that generates audio (may be *infinite*, but is not
/// required).
pub trait Stream<F: Frame>: Sized + IntoIterator<Item = F> {
    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> Option<f64>;

    /// Set the source sample rate of the stream.  Will usually panic (default
    /// behavior), unless the stream is configurable.
    fn set_sample_rate(&mut self, _sr: f64) {
        panic!("set_sample_rate() called on a fixed-sample rate stream!")
    }

    /// Returns the length of the stream exactly.  `None` represents an infinite
    /// iterator.
    fn len(&self) -> Option<usize>;
}
