// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{ops::Blend, Frame};
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
    offseti: f32,
}

impl<F: Frame> Resampler<F> {
    /// Create a new resampler context.
    pub fn new(frame: F, index: f32) -> Self {
        Self {
            partial: frame,
            offseti: index,
        }
    }

    /// Get the left over partial frame.
    pub fn frame(&self) -> F {
        self.partial
    }

    /// Get the left over partial index.
    pub fn index(&self) -> f32 {
        self.offseti
    }
}

/// Audio sink - a type that consumes audio samples.
pub trait Sink<F: Frame>: Sized {
    /// Get the (target) sample rate of the [`Sink`](crate::Sink).
    fn sample_rate(&self) -> u32;

    /// Get the [`Resampler`](crate::Resampler) context for this
    /// [`Sink`](crate::Sink).
    fn resampler(&mut self) -> &mut Resampler<F>;

    /// Get the (target) audio [Frame](crate::Frame) buffer of the
    /// [`Sink`](crate::Sink).
    fn buffer(&mut self) -> &mut [F];

    /// Flush the partial sample from the resampler into the audio buffer if
    /// there is one.
    fn flush(mut self) {
        if self.resampler().offseti % 1.0 > f32::EPSILON
            || self.resampler().offseti % 1.0 < -f32::EPSILON
        {
            let i = self.resampler().offseti as usize;
            self.buffer()[i] = self.resampler().partial;
        }
    }

    /// [`Stream`](crate::Stream) audio into this audio [`Sink`](crate::Sink).
    #[inline(always)]
    fn stream<S: Frame, M: Stream<S>>(&mut self, mut stream: M) {
        // Ratio of destination samples per stream samples.
        let ratio = if let Some(stream_sr) = stream.sample_rate() {
            self.sample_rate() as f32 / stream_sr as f32
        } else {
            stream.set_sample_rate(self.sample_rate());
            1.0
        };
        // Add left over audio.
        let partial = self.resampler().partial;
        if let Some(dst) = self.buffer().get_mut(0) {
            *dst = *dst + partial;
        }
        // Calculate Ranges
        let mut srclen = stream.len();
        let dst_range = if let Some(len) = stream.len() {
            ..((ratio * len as f32) as usize).min(self.buffer().len())
        } else {
            ..self.buffer().len()
        };
        // Clear destination range.
        for f in self.buffer()[dst_range].iter_mut() {
            *f = F::default();
        }
        // Go through each source sample and add to destination.
        let mut stream_iter = stream.into_iter();
        for i in 0.. {
            // Calculate destination index.
            let j = ratio * i as f32 + self.resampler().offseti;
            let ceil = j.ceil() as usize;
            let floor = j as usize;
            if !dst_range.contains(&floor) {
                srclen = Some(i);
                break;
            }
            let ceil_f64 = (j % 1.0).min(ratio);
            let ceil_a = F::from(ceil_f64);
            let floor_a = F::from(ratio - ceil_f64);
            let src = if let Some(src) = stream_iter.next() {
                src
            } else {
                break;
            };
            let src: F = src.convert();
            self.buffer()[dst_range][floor] = self.buffer()[dst_range][floor] + src * floor_a;
            if let Some(buf) = self.buffer()[dst_range].get_mut(ceil) {
                *buf = *buf + src * ceil_a;
            } else {
                self.resampler().partial = self.resampler().partial + src * ceil_a;
            }
        }
        // Increment offseti
        self.resampler().offseti += ratio * srclen.unwrap() as f32;
    }
}

/// Audio stream - a type that generates audio samples.
pub trait Stream<F: Frame>: Sized + IntoIterator<Item = F> {
    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> Option<u32>;

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
    fn blend<G: Frame, M: Stream<G>, O: Blend>(
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
    fn sample_rate(&self) -> Option<u32> {
        self.0.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> Option<usize> {
        Some(self.0.len().map(|a| a.min(self.1)).unwrap_or(self.1))
    }

    #[inline(always)]
    fn set_sample_rate<R: Into<f64>>(&mut self, rate: R) {
        self.0.set_sample_rate(rate)
    }
}

/// Blended stream.
#[derive(Debug)]
pub struct BlendStream<F, G, A, B, O>(A, B, PhantomData<(F, G, O)>)
where
    F: Frame,
    G: Frame,
    A: Stream<F>,
    B: Stream<G>,
    O: Blend;

impl<F, G, A, B, O> IntoIterator for BlendStream<F, G, A, B, O>
where
    F: Frame,
    G: Frame,
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
            .map(|(a, b)| O::mix_frames(a, b.convert()))
    }
}

impl<F, G, A, B, O> Stream<F> for BlendStream<F, G, A, B, O>
where
    F: Frame,
    G: Frame,
    A: Stream<F>,
    B: Stream<G>,
    O: Blend,
{
    #[inline(always)]
    fn sample_rate(&self) -> Option<u32> {
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
