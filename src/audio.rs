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
    chan::{Ch16, Ch32, Ch64, Ch8, Channel},
    mono::{Mono, Mono64},
    ops::Blend,
    Frame, Resampler, Sink, Stream,
};
use core::{
    fmt::Debug,
    ops::{Bound::*, RangeBounds},
    slice::{from_raw_parts_mut, SliceIndex},
};

// Channel Identification
// 0. Front Left (Mono)
// 1. Front Right
// 2. Center
// 3. Rear Left
// 4. Rear Right
// 5. LFE
// 6. Side Left
// 7. Side Right

/// Audio buffer (array of audio `Sample`s at sample rate specified in hertz).
#[derive(Debug)]
pub struct Audio<F: Frame> {
    s_rate: u32,
    frames: Box<[F]>,
}

impl<F: Frame> Audio<F> {
    /// Get an audio frame.
    ///
    /// # Panics
    /// If index is out of bounds
    pub fn get(&self, index: usize) -> &F {
        self.frames.get(index).expect("Sample out of bounds")
    }

    /// Get a mutable reference to an audio frame.
    ///
    /// # Panics
    /// If index is out of bounds
    pub fn get_mut(&mut self, index: usize) -> &mut F {
        self.frames.get_mut(index).expect("Sample out of bounds")
    }

    /// Get a slice of all audio frames.
    pub fn as_slice(&self) -> &[F] {
        &self.frames
    }

    /// Get a mutable slice of all audio frames.
    pub fn as_mut_slice(&mut self) -> &mut [F] {
        &mut self.frames
    }

    /// Returns an iterator over the audio frames.
    pub fn iter(&self) -> std::slice::Iter<'_, F> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, F> {
        self.frames.iter_mut()
    }

    /// Construct an `Audio` buffer with all audio frames set to one value.
    pub fn with_frame(s_rate: u32, len: usize, frame: F) -> Self {
        let frames = vec![frame; len].into_boxed_slice();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    pub fn with_silence(s_rate: u32, len: usize) -> Self {
        Self::with_frame(s_rate, len, F::default())
    }

    /// Construct an `Audio` buffer with another `Audio` buffer.
    ///
    /// The audio format can be converted with this function.
    pub fn with_audio<S: Frame>(s_rate: u32, src: &Audio<S>) -> Self
    where
        F::Chan: From<S::Chan>,
    {
        // FIXME: Use Stream and Sink traits.

        let src_sr = src.sample_rate();

        // Check if resampling can be skipped.
        if src_sr == s_rate {
            let mut dst = Audio::with_silence(src_sr, src.len());

            for (dst, src) in dst.iter_mut().zip(src.iter()) {
                *dst = src.convert();
            }

            return dst;
        }

        // Calculate ratio of how many destination samples per source samples.
        let sr_rat = s_rate as f64 / src_sr as f64;
        // Calculate total number of samples for destination.
        let dstlen = (sr_rat * src.len() as f64) as usize;
        // Calculate the index multiplier.
        let src_per_dst = (src.len() - 1) as f64 / (dstlen - 1) as f64;
        // Generate silence for destination.
        let mut dst = Audio::with_silence(s_rate, dstlen);

        // Go through each destination sample and interpolate from source.
        // FIXME: Optimize?
        for (i, dst) in dst.iter_mut().enumerate() {
            // Get index in source.
            let j = i as f64 * src_per_dst;
            // Get floor at source index.
            let floor: F = src.get(j.floor() as usize).convert();
            // Get ceiling at source index.
            let ceil: F = src.get(j.ceil() as usize).convert();
            // Get interpolation amount.
            let amt = j % 1.0;

            // Interpolate between the samples.
            *dst = floor.lerp(ceil, Mono64::new(Ch64::from(amt)).convert());
        }

        //

        dst
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_frames<B: Into<Box<[F]>>>(s_rate: u32, frames: B) -> Self {
        let frames = frames.into();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `i8` buffer.
    #[allow(unsafe_code)]
    pub fn with_i8_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i8]>>,
        F: Frame<Chan = Ch8>,
    {
        let buffer: Box<[i8]> = buffer.into();
        let len = buffer.len() / std::mem::size_of::<F>();
        assert_eq!(0, buffer.len() % std::mem::size_of::<F>());
        let slice = Box::<[i8]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
        F: Frame<Chan = Ch16>,
    {
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<i16>();
        let len = bytes / std::mem::size_of::<F>();
        assert_eq!(0, bytes % std::mem::size_of::<F>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
        F: Frame<Chan = Ch32>,
    {
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<f32>();
        let len = bytes / std::mem::size_of::<F>();
        assert_eq!(0, bytes % std::mem::size_of::<F>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `f64` buffer.
    #[allow(unsafe_code)]
    pub fn with_f64_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f64]>>,
        F: Frame<Chan = Ch64>,
    {
        let buffer: Box<[f64]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<f64>();
        let len = bytes / std::mem::size_of::<F>();
        assert_eq!(0, bytes % std::mem::size_of::<F>());
        let slice = Box::<[f64]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Get the length of the `Audio` buffer.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Check if `Audio` buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the sample rate of the `Audio` buffer.
    pub fn sample_rate(&self) -> u32 {
        self.s_rate
    }

    /// Create an audio stream over this `Audio` buffer.
    ///
    /// # Panics
    /// If range is out of bounds
    pub fn stream<'a, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + 'a>(
        &'a self,
        reg: R,
    ) -> impl Stream<F> + 'a {
        assert!(reg.end_bound() == Unbounded || !reg.contains(&self.len()));
        AudioStream {
            resampler: Resampler::new(),
            cursor: match reg.start_bound() {
                Unbounded => 0,
                Included(index) => *index,
                Excluded(index) => *index + 1,
            },
            range: reg,
            audio: self,
        }
    }

    /// Create a draining audio stream over this `Audio` buffer.
    ///
    /// # Panics
    /// If range is out of bounds
    pub fn drain<'a, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone + 'a>(
        &'a mut self,
        reg: R,
    ) -> impl Stream<F> + 'a {
        assert!(reg.end_bound() == Unbounded || !reg.contains(&self.len()));
        let index = match reg.start_bound() {
            Unbounded => 0,
            Included(index) => *index,
            Excluded(index) => *index + 1,
        };
        AudioDrain {
            resampler: Resampler::new(),
            cursor: index,
            start: index,
            range: reg,
            audio: self,
        }
    }

    /// Create an audio sink to overwrite this `Audio` buffer.
    ///
    /// # Panics
    /// If range is out of bounds
    pub fn sink<'a, R: 'a + RangeBounds<usize> + SliceIndex<[F], Output = [F]>>(
        &'a mut self,
        reg: R,
    ) -> impl Sink + 'a {
        assert!(reg.end_bound() == Unbounded || !reg.contains(&self.len()));
        AudioSink {
            cursor: match reg.start_bound() {
                Unbounded => 0,
                Included(index) => *index,
                Excluded(index) => *index + 1,
            },
            range: reg,
            audio: self,
        }
    }

    /// Extend the audio buffer with all of the audio from a stream.
    ///
    /// *Don't call this with an infinite stream!*  This is the only way to
    /// collect a stream without knowing the size ahead of time.
    pub fn extend<M: Stream<F>>(&mut self, stream: &mut M) {
        let mut audio: Box<[F]> = Vec::new().into();
        std::mem::swap(&mut audio, &mut self.frames);
        let mut audio: Vec<F> = audio.into();
        while let Some(sample) = stream.stream_sample() {
            audio.push(sample);
        }
        let mut audio: Box<[F]> = audio.into();
        std::mem::swap(&mut audio, &mut self.frames);
    }
}

impl<F: Frame<Chan = Ch8>> Audio<F> {
    /// Get view of samples as an `i8` slice.
    pub fn as_i8_slice(&self) -> &[i8] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to::<i8>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }

    /// Get mutable view of samples as a mutable `i8` slice.
    pub fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to_mut::<i8>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<F: Frame<Chan = Ch16>> Audio<F> {
    /// Get view of samples as an `i16` slice.
    pub fn as_i16_slice(&self) -> &[i16] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }

    /// Get mutable view of samples as a mutable `i16` slice.
    pub fn as_i16_slice_mut(&mut self) -> &mut [i16] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to_mut::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<S: Frame<Chan = Ch32>> Audio<S> {
    /// Get view of samples as an `f32` slice.
    pub fn as_f32_slice(&self) -> &[f32] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to::<f32>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }

    /// Get mutable view of samples as a mutable `f32` slice.
    pub fn as_f32_slice_mut(&mut self) -> &mut [f32] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to_mut::<f32>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<F: Frame<Chan = Ch64>> Audio<F> {
    /// Get view of samples as an `f64` slice.
    pub fn as_f64_slice(&self) -> &[f64] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to::<f64>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }

    /// Get mutable view of samples as a mutable `f64` slice.
    pub fn as_f64_slice_mut(&mut self) -> &mut [f64] {
        unsafe {
            let (prefix, v, suffix) = self.frames.align_to_mut::<f64>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

/// A `Sink` created with `Audio.sink()`
struct AudioSink<'a, F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> {
    audio: &'a mut Audio<F>,
    cursor: usize,
    range: R,
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> Sink for AudioSink<'_, F, R> {
    fn sample_rate(&self) -> u32 {
        self.audio.sample_rate()
    }

    fn sink_sample<O: Blend, Z: Frame>(&mut self, sample: Z, op: O) {
        // If empty, start over
        if !self.range.contains(&self.cursor) {
            self.cursor = match self.range.start_bound() {
                Unbounded => 0,
                Included(index) => *index,
                Excluded(index) => *index + 1,
            };
        }
        self.audio.get_mut(self.cursor).blend(&sample.convert(), op);
        self.cursor += 1;
    }

    fn sink_sample_panned<O: Blend, C: Channel>(&mut self, sample: Mono<C>, op: O, pan: f64) {
        // If empty, start over
        if !self.range.contains(&self.cursor) {
            self.cursor = match self.range.start_bound() {
                Unbounded => 0,
                Included(index) => *index,
                Excluded(index) => *index + 1,
            };
        }
        self.audio
            .get_mut(self.cursor)
            .blend(&F::from_mono_panned(sample.convert(), pan), op);
        self.cursor += 1;
    }

    fn capacity(&self) -> usize {
        self.audio.len()
    }
}

/// A `Stream` created with `Audio.stream()`
struct AudioStream<'a, F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> {
    resampler: Resampler<F>,
    audio: &'a Audio<F>,
    cursor: usize,
    range: R,
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> Stream<F>
    for AudioStream<'_, F, R>
{
    fn sample_rate(&self) -> u32 {
        self.audio.sample_rate()
    }

    fn stream_sample(&mut self) -> Option<F> {
        if
        /* is empty */
        !self.range.contains(&self.cursor) {
            return None;
        }
        let sample = self.audio.get(self.cursor);
        self.cursor += 1;
        Some(*sample)
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.resampler
    }
}

/// A `Stream` created with `Audio.drain()`
struct AudioDrain<'a, F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone> {
    resampler: Resampler<F>,
    audio: &'a mut Audio<F>,
    cursor: usize,
    start: usize, // Where the cursor starts
    range: R,
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone> Stream<F>
    for AudioDrain<'_, F, R>
{
    fn sample_rate(&self) -> u32 {
        self.audio.sample_rate()
    }

    fn stream_sample(&mut self) -> Option<F> {
        if
        /* is empty */
        !self.range.contains(&self.cursor) || self.cursor == self.audio.len() {
            return None;
        }
        let frame = self.audio.get(self.cursor);
        self.cursor += 1;
        Some(*frame)
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.resampler
    }
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone> Drop
    for AudioDrain<'_, F, R>
{
    fn drop(&mut self) {
        let mut audio: Box<[F]> = Vec::new().into();
        std::mem::swap(&mut audio, &mut self.audio.frames);
        let mut audio: Vec<F> = audio.into();
        audio.drain(self.start..self.cursor);
        let mut audio: Box<[F]> = audio.into();
        std::mem::swap(&mut audio, &mut self.audio.frames);
    }
}

impl<F: Frame> From<Audio<F>> for Box<[F]> {
    /// Get internal sample data as a boxed slice of audio frames.
    fn from(audio: Audio<F>) -> Self {
        audio.frames
    }
}

impl<F: Frame> From<Audio<F>> for Vec<F> {
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<F>) -> Self {
        audio.frames.into()
    }
}

impl<F: Frame<Chan = Ch8>> From<Audio<F>> for Box<[i8]> {
    /// Get internal sample data as boxed slice of *i8*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let frames = audio.frames;
        let capacity = frames.len() * std::mem::size_of::<F>();
        let slice = Box::<[F]>::into_raw(frames);
        let buffer: Box<[i8]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut i8;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<F: Frame<Chan = Ch16>> From<Audio<F>> for Box<[i16]> {
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let frames = audio.frames;
        let capacity = frames.len() * std::mem::size_of::<F>() / 2;
        let slice = Box::<[F]>::into_raw(frames);
        let buffer: Box<[i16]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<F: Frame<Chan = Ch32>> From<Audio<F>> for Box<[f32]> {
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let frames = audio.frames;
        let capacity = frames.len() * std::mem::size_of::<F>() / 4;
        let slice = Box::<[F]>::into_raw(frames);
        let buffer: Box<[f32]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<F: Frame<Chan = Ch64>> From<Audio<F>> for Box<[f64]> {
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let frames = audio.frames;
        let capacity = frames.len() * std::mem::size_of::<F>() / 8;
        let slice = Box::<[F]>::into_raw(frames);
        let buffer: Box<[f64]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
