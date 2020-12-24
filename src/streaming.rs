// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::{math, ops::Blend, Frame};
use core::{
    iter::{Map, Take, Zip},
    marker::PhantomData,
};

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
            *dst += partial;
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

/// Audio stream - a type that generates audio samples.
pub trait Stream<F: Frame>: Sized + IntoIterator<Item = F> {
    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> Option<f64>;

    /// Returns the length of the stream exactly.  `None` represents an infinite
    /// iterator.
    fn len(&self) -> Option<usize>;

    /// Check if the stream is empty (will not produce any frames).
    fn is_empty(&self) -> bool {
        self.len() == Some(0)
    }

    /// Set the source sample rate of the stream.  Will usually panic (default
    /// behavior), unless the stream is configurable.
    fn set_sample_rate<R: Into<f64>>(&mut self, sr: R) {
        let sr = sr.into();
        panic!(
            "set_sample_rate({}) called on a fixed-sample rate stream!",
            sr
        )
    }

    /// Take at most `samples` samples as a stream.
    fn take(self, samples: usize) -> TakeStream<F, Self> {
        TakeStream(self, samples, PhantomData)
    }

    /// Blend this stream with another.
    ///
    /// # Panics
    /// If the sample rates are not compatible.
    fn blend<G: Frame + Into<F>, M: Stream<G>, O: Blend>(
        self,
        other: M,
        op: O,
    ) -> BlendStream<F, G, Self, M, O> {
        let mut first = self;
        let mut second = other;

        let _op = op;
        let (sr_a, sr_b) = (first.sample_rate(), second.sample_rate());
        if sr_a != sr_b {
            assert!(sr_a.is_none() || sr_b.is_none());
            match (sr_a, sr_b) {
                (None, None) => { /* Do nothing */ }
                (None, Some(sr)) => first.set_sample_rate(sr),
                (Some(sr), None) => second.set_sample_rate(sr),
                (Some(_), Some(_)) => unreachable!(),
            }
        }
        BlendStream(first, second, PhantomData)
    }
}

/// Take stream.
#[derive(Debug)]
pub struct TakeStream<F: Frame, S: Stream<F>>(S, usize, PhantomData<F>);

impl<F: Frame, S: Stream<F>> IntoIterator for TakeStream<F, S> {
    type Item = F;
    type IntoIter = Take<S::IntoIter>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().take(self.1)
    }
}

impl<F: Frame, S: Stream<F>> Stream<F> for TakeStream<F, S> {
    #[inline(always)]
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> Option<usize> {
        self.0.len().map(|a| a.min(self.1))
    }
}

/// Blended stream.
#[derive(Debug)]
pub struct BlendStream<F, G, A, B, O>(A, B, PhantomData<(F, G, O)>)
where
    F: Frame,
    G: Frame + Into<F>,
    A: Stream<F>,
    B: Stream<G>,
    O: Blend;

impl<F, G, A, B, O> IntoIterator for BlendStream<F, G, A, B, O>
where
    F: Frame,
    G: Frame + Into<F>,
    A: Stream<F>,
    B: Stream<G>,
    O: Blend,
{
    type Item = F;
    #[allow(clippy::type_complexity)]
    type IntoIter = Map<Zip<A::IntoIter, B::IntoIter>, fn((F, G)) -> F>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .zip(self.1.into_iter())
            .map(|(a, b)| O::compose(a, b.into()))
    }
}

impl<F, G, A, B, O> Stream<F> for BlendStream<F, G, A, B, O>
where
    F: Frame,
    G: Frame + Into<F>,
    A: Stream<F>,
    B: Stream<G>,
    O: Blend,
{
    #[inline(always)]
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> Option<usize> {
        match (self.0.len(), self.1.len()) {
            (None, None) => None,
            (None, Some(len)) => Some(len),
            (Some(len), None) => Some(len),
            (Some(a), Some(b)) => Some(a.min(b)),
        }
    }

    #[inline(always)]
    fn set_sample_rate<R: Into<f64>>(&mut self, sr: R) {
        let sr = sr.into();
        self.0.set_sample_rate(sr);
        self.1.set_sample_rate(sr);
    }
}
