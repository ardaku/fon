use core::{fmt::Debug, num::NonZeroU32};

use crate::{chan::Channel, Frame};

/// Audio sink - a type that consumes audio frames.
pub trait Sink<C: Channel, const COUNT: usize>: Debug {
    /// Get the sample rate of the sink in hertz.
    fn sample_rate(&self) -> NonZeroU32;

    /// Get the length of the sink in frames.
    ///
    /// Sinks must always have finite length.
    fn len(&self) -> usize;

    /// Sink audio frames from a frame iterator.
    ///
    /// **Warning**: if used incorrectly, this method may introduce audio
    /// aliasing.  To avoid that, make sure the sample rate of the frames from
    /// the iterator matches exactly the sample rate of the sink.
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<C, COUNT>>);

    /// Check if the sink is empty (length of zero).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Sink that converts to a different audio format before passing to another
/// [`Sink`](crate::Sink).
#[derive(Debug)]
pub struct SinkTo<C, F, K, const COUNT: usize, const N: usize>
where
    C: Channel + From<F>,
    F: Channel,
    K: Sink<C, COUNT>,
{
    sink: K,
    _phantom: core::marker::PhantomData<fn() -> (C, F)>,
}

impl<C, F, K, const COUNT: usize, const N: usize> SinkTo<C, F, K, COUNT, N>
where
    C: Channel + From<F>,
    F: Channel,
    K: Sink<C, COUNT>,
{
    /// Convert an arbitrary `Sink` type to a different format.
    pub fn new(sink: K) -> Self {
        Self {
            sink,
            _phantom: core::marker::PhantomData,
        }
    }
}

#[allow(single_use_lifetimes)]
impl<C, F, K, const COUNT: usize, const N: usize> Sink<F, N>
    for &mut SinkTo<C, F, K, COUNT, N>
where
    C: Channel + From<F>,
    F: Channel,
    K: Sink<C, COUNT>,
{
    /// Get the sample rate of the sink in hertz.
    fn sample_rate(&self) -> NonZeroU32 {
        self.sink.sample_rate()
    }

    /// Get the length of the sink in frames.
    ///
    /// Sinks must always have finite length.
    fn len(&self) -> usize {
        self.sink.len()
    }

    /// Sink audio samples from a frame iterator.
    ///
    /// **Warning**: if used incorrectly, this method may introduce audio
    /// aliasing.  To avoid that, make sure the sample rate of the frames from
    /// the iterator matches exactly the sample rate of the sink.
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<F, N>>) {
        self.sink.sink_with(&mut iter.map(Frame::to))
    }
}

#[allow(single_use_lifetimes)]
impl<C, F, K, const COUNT: usize, const N: usize> Sink<F, N>
    for SinkTo<C, F, K, COUNT, N>
where
    C: Channel + From<F>,
    F: Channel,
    K: Sink<C, COUNT>,
{
    /// Get the sample rate of the sink in hertz.
    fn sample_rate(&self) -> NonZeroU32 {
        self.sink.sample_rate()
    }

    /// Get the length of the sink in frames.
    ///
    /// Sinks must always have finite length.
    fn len(&self) -> usize {
        self.sink.len()
    }

    /// Sink audio samples from a frame iterator.
    ///
    /// **Warning**: if used incorrectly, this method may introduce audio
    /// aliasing.  To avoid that, make sure the sample rate of the frames from
    /// the iterator matches exactly the sample rate of the sink.
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<F, N>>) {
        self.sink.sink_with(&mut iter.map(Frame::to))
    }
}
