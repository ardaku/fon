// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::{
    chan::{Ch64, Channel},
    mono::Mono,
    ops::Blend,
    sample::Sample,
};

/// Context for an audio resampler.
#[derive(Default, Debug, Copy, Clone)]
pub struct Resampler<S: Sample> {
    /// How much of a sample is represented by `part`
    phase: f64,
    /// A last sample read
    part: S,
}

impl<S: Sample> Resampler<S> {
    /// Create a new resampler context.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Audio sink - a type that consumes a *finite* number of audio samples.
pub trait Sink: Sized {
    /// Transfer the audio from a `Stream` into a `Sink`.
    fn sink<O: Blend, Z: Sample, M: Stream<Z>>(&mut self, stream: &mut M, op: O) {
        stream.stream(self, op)
    }

    /// Transfer the audio from a `Stream` panned left or right into a `Sink`.
    ///
    /// Pan of 0 is front, negative pan for left (-0.5 is back), and positive
    /// pan for right (0.5 is also back).  Values outside the range will wrap.
    fn sink_panned<O: Blend, C: Channel, M: Stream<Mono<C>>>(
        &mut self,
        stream: &mut M,
        op: O,
        pan: f64,
    ) {
        // Silence
        let zero = Mono::new(C::MID);

        // Faster algorithm if sample rates match.
        if stream.sample_rate() == self.sample_rate() {
            for _ in 0..self.capacity() {
                self.sink_sample(stream.stream_sample().unwrap_or(zero), op)
            }
            return;
        }

        // How many samples to read for each sample written.
        let sr_ratio = stream.sample_rate() as f64 / self.sample_rate() as f64;

        // Write into the entire capacity of the `Sink`.
        for _ in 0..self.capacity() {
            let old_phase = stream.resampler().phase;
            // Increment phase
            stream.resampler().phase += sr_ratio;

            if stream.resampler().phase >= 1.0 {
                // Value is always overwritten, but Rust compiler can't prove it
                let mut sample = zero;
                // Read one or more samples to interpolate & write out
                while stream.resampler().phase >= 1.0 {
                    sample = stream.stream_sample().unwrap_or(zero);
                    stream.resampler().phase = stream.resampler().phase - 1.0;
                    stream.resampler().part = sample;
                }
                let amount = Mono::new::<C>(Ch64::new(old_phase).into());
                let sample = stream.resampler().part.lerp(sample, amount);
                self.sink_sample_panned(sample, op, pan);
            } else {
                // Don't read any samples - copy & write the last one
                self.sink_sample_panned(stream.resampler().part, op, pan);
            }
        }
    }

    /// Get the (target) sample rate of the sink.
    fn sample_rate(&self) -> u32;

    /// This function is called when the sink receives a sample from a stream.
    fn sink_sample<O: Blend, Z: Sample>(&mut self, sample: Z, op: O);
    
    /// This function is called when the sink receives a sample from a stream.
    fn sink_sample_panned<O: Blend, C: Channel>(&mut self, sample: Mono<C>, op: O, pan: f64);

    /// Get the (target) capacity of the sink.  Returns the number of times it's
    /// permitted to call `sink_sample()`.  Additional calls over capacity
    /// should wrap around to the beginning of the buffer.
    fn capacity(&self) -> usize;
}

/// Audio stream - a type that generates audio (may be *infinite*, but is not
/// required).
pub trait Stream<S: Sample>: Sized {
    /// Transfer the audio from a `Stream` into a `Sink`.  You may write to the
    /// same sink multiple times, blending audio with `Blend` operations.
    fn stream<O: Blend, K: Sink>(&mut self, sink: &mut K, op: O) {
        // Silence
        let zero = Mono::<S::Chan>::new::<S::Chan>(S::Chan::MID).convert();

        // Faster algorithm if sample rates match.
        if self.sample_rate() == sink.sample_rate() {
            for _ in 0..sink.capacity() {
                sink.sink_sample(self.stream_sample().unwrap_or(zero), op)
            }
            return;
        }

        // How many samples to read for each sample written.
        let sr_ratio = self.sample_rate() as f64 / sink.sample_rate() as f64;

        // Write into the entire capacity of the `Sink`.
        for _ in 0..sink.capacity() {
            let old_phase = self.resampler().phase;
            // Increment phase
            self.resampler().phase += sr_ratio;

            if self.resampler().phase >= 1.0 {
                // Value is always overwritten, but Rust compiler can't prove it
                let mut sample = zero;
                // Read one or more samples to interpolate & write out
                while self.resampler().phase >= 1.0 {
                    sample = self.stream_sample().unwrap_or(zero);
                    self.resampler().phase = self.resampler().phase - 1.0;
                    self.resampler().part = sample;
                }
                let amount = Mono::<S::Chan>::new(Ch64::new(old_phase)).convert();
                let sample = self.resampler().part.lerp(sample, amount);
                sink.sink_sample(sample, op);
            } else {
                // Don't read any samples - copy & write the last one
                sink.sink_sample(self.resampler().part, op);
            }
        }
    }

    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> u32;

    /// This function is called when a sink requests a sample from the stream.
    fn stream_sample(&mut self) -> Option<S>;

    /// Get this streams's resampler context.
    fn resampler(&mut self) -> &mut Resampler<S>;
}
