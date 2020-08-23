// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    chan::Channel,
    sample::{Sample, Sample1},
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

/// Audio sink - a type that consumes audio (as much as the audio `Stream` can
/// produce).
pub trait Sink<S: Sample>: Sized {
    /// Transfer audio from a stream into the sink.
    fn sink<M: Stream<S>>(&mut self, stream: &mut M) {
        stream.stream(self)
    }

    /// Get the (target) sample rate of the sink.
    fn sample_rate(&self) -> u32;

    /// This function is called when the sink receives a sample from a stream.
    fn sink_sample(&mut self, sample: S);
}

/// Audio stream - a type that generates a *finite* amount of audio.
pub trait Stream<S: Sample>: Sized {
    /// Transfer the audio from a stream into the sink.
    fn stream<K: Sink<S>>(&mut self, sink: &mut K) {
        // How many samples to write for each sample read.
        let sr_ratio = sink.sample_rate() as f64 / self.sample_rate() as f64;
        // Go through each sample.
        while let Some(sample) = self.stream_sample() {
            let sample = *sample;
            let old_phase = self.resampler().phase;
            // Increment phase
            self.resampler().phase += sr_ratio;
            // If an in-between sample should be sinked.
            if self.resampler().phase >= 1.0 {
                let amount = Sample1::<S::Chan>::new(old_phase).convert();
                let sample = self.resampler().part.lerp(sample, amount);
                sink.sink_sample(sample);
                self.resampler().part = sample;
                self.resampler().phase -= 1.0;
            }
            // Copied samples.
            while self.resampler().phase >= 1.0 {
                sink.sink_sample(sample);
                self.resampler().phase -= 1.0;
            }
        }
    }

    /// Flush any partially resampled sample (mix with silence), and push out
    /// to the audio `Sink`.
    fn flush<K: Sink<S>>(&mut self, sink: &mut K) {
        let middle = Sample1::<S::Chan>::new::<S::Chan>(S::Chan::MID).convert();
        let amount = Sample1::<S::Chan>::new(self.resampler().phase).convert();
        let sample = self.resampler().part.lerp(middle, amount);
        sink.sink_sample(sample);
        *self.resampler() = Resampler::default();
    }

    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> u32;

    /// This function is called when a sink requests a sample from the stream.
    fn stream_sample(&mut self) -> Option<&S>;

    /// Get this streams's resampler context.
    fn resampler(&mut self) -> &mut Resampler<S>;
}
