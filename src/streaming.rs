// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::sample::Sample;

/// Context for an audio resampler.
#[derive(Default, Debug, Copy, Clone)]
pub struct Resampler {
    /// How much of a sample is represented by `part`
    phase: f64,
    /// A partial sample
    part: f64,
}

impl Resampler {
    /// Create a new resampler context.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Audio sink - a type that consumes audio.
pub trait Sink<S: Sample> {
    /// Transfer audio from a stream into the sink.
    fn sink<M: Stream<S>>(&mut self, stream: &mut M) {
        
    }

    /// Get the (target) sample rate of the sink.
    fn sample_rate(&self) -> u32;

    /// This function is called when the sink receives a sample from a stream.
    fn sink_sample(&mut self, sample: S);

    /// Get this sink's resampler context.
    fn resampler(&mut self) -> &mut Resampler;
}

/// Audio stream - a type that generates audio.
pub trait Stream<S: Sample>: Sized {
    /// Transfer audio from a stream into the sink.
    fn stream<K: Sink<S>>(&mut self, sink: &mut K) {
        sink.sink(self)
    }

    /// Get the (source) sample rate of the stream.
    fn sample_rate(&self) -> u32;

    /// This function is called when a sink requests a sample from the stream.
    fn stream_sample(&mut self) -> Option<&S>;
}
