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
    chan::{Ch16, Ch32, Ch64, Ch8},
    math, Frame, Resampler, Sink, Stream,
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::Debug,
    iter::Cloned,
    mem::{size_of, swap},
    ops::{Bound::*, RangeBounds},
    slice::{from_raw_parts_mut, Iter, IterMut, SliceIndex},
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

/// Audio buffer (array of audio [`Frame`](crate::Frame)s at sample rate
/// specified in hertz).
///
/// `Audio` implements the [`Stream`](crate::Stream) trait.
#[derive(Debug)]
pub struct Audio<F: Frame> {
    s_rate: f64,
    frames: Box<[F]>,
}

impl<F: Frame> Audio<F> {
    /// Get an audio frame.
    pub fn get(&self, index: usize) -> Option<F> {
        self.frames.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut F> {
        self.frames.get_mut(index)
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
    pub fn iter(&self) -> Iter<'_, F> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    pub fn iter_mut(&mut self) -> IterMut<'_, F> {
        self.frames.iter_mut()
    }

    /// Construct an `Audio` buffer with all audio frames set to one value.
    pub fn with_frame<R: Into<f64>>(s_rate: R, len: usize, frame: F) -> Self {
        let s_rate = s_rate.into();
        let frames = vec![frame; len].into_boxed_slice();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    pub fn with_silence<R: Into<f64>>(s_rate: R, len: usize) -> Self {
        Self::with_frame(s_rate, len, F::default())
    }

    /// Construct an [`Audio`](crate::Audio) buffer from the contents of a
    /// [`Stream`](crate::Stream).
    ///
    /// The audio format can be converted with this function.
    ///
    /// # Panics
    /// When an infinite stream is passed in.
    pub fn with_stream<S, R, M>(s_rate: R, src: M) -> Self
    where
        F::Chan: From<S::Chan>,
        R: Into<f64>,
        M: Stream<S>,
        S: Frame,
    {
        let s_rate = s_rate.into();
        let mut audio = Self::with_frames::<[F; 0], f64>(s_rate, []);
        audio.extend(src);
        audio
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_frames<B: Into<Box<[F]>>, R: Into<f64>>(
        s_rate: R,
        frames: B,
    ) -> Self {
        let s_rate = s_rate.into();
        let frames = frames.into();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `i8` buffer.
    #[allow(unsafe_code)]
    pub fn with_i8_buffer<B, R>(s_rate: R, buffer: B) -> Self
    where
        B: Into<Box<[i8]>>,
        F: Frame<Chan = Ch8>,
        R: Into<f64>,
    {
        let s_rate = s_rate.into();
        let buffer: Box<[i8]> = buffer.into();
        let len = buffer.len() / size_of::<F>();
        assert_eq!(0, buffer.len() % size_of::<F>());
        let slice = Box::<[i8]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B, R>(s_rate: R, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
        F: Frame<Chan = Ch16>,
        R: Into<f64>,
    {
        let s_rate = s_rate.into();
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<F>();
        assert_eq!(0, bytes % size_of::<F>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B, R>(s_rate: R, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
        F: Frame<Chan = Ch32>,
        R: Into<f64>,
    {
        let s_rate = s_rate.into();
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * size_of::<f32>();
        let len = bytes / size_of::<F>();
        assert_eq!(0, bytes % size_of::<F>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer from an `f64` buffer.
    #[allow(unsafe_code)]
    pub fn with_f64_buffer<B, R>(s_rate: R, buffer: B) -> Self
    where
        B: Into<Box<[f64]>>,
        F: Frame<Chan = Ch64>,
        R: Into<f64>,
    {
        let s_rate = s_rate.into();
        let buffer: Box<[f64]> = buffer.into();
        let bytes = buffer.len() * size_of::<f64>();
        let len = bytes / size_of::<F>();
        assert_eq!(0, bytes % size_of::<F>());
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
    pub fn sample_rate(&self) -> f64 {
        self.s_rate
    }

    /// Create a draining audio stream over this `Audio` buffer.
    ///
    /// # Panics
    /// If range is out of bounds
    pub fn drain<
        'a,
        R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone + 'a,
    >(
        &'a mut self,
        reg: R,
    ) -> impl Stream<F> + '_ {
        assert!(reg.end_bound() == Unbounded || !reg.contains(&self.len()));

        // Convert audio into a Vec.
        let mut temp_move = Self::with_frames::<[F; 0], f64>(self.s_rate, []);
        swap(self, &mut temp_move);
        let audio: Box<[F]> = temp_move.into();
        let audio: Vec<F> = audio.into();
        let cursor = match reg.start_bound() {
            Included(a) => *a,
            Excluded(a) => *a + 1,
            Unbounded => 0,
        };

        AudioDrain {
            audio,
            region: reg,
            cursor,
            buffer: self,
        }
    }

    /// Create an audio sink to overwrite this `Audio` buffer.
    ///
    /// # Panics
    /// If range is out of bounds
    pub fn sink<
        'a,
        R: 'a + RangeBounds<usize> + SliceIndex<[F], Output = [F]>,
    >(
        &'a mut self,
        reg: R,
    ) -> impl Sink<F> + '_ {
        assert!(reg.end_bound() == Unbounded || !reg.contains(&self.len()));
        AudioSink {
            s_rate: self.sample_rate(),
            frames: &mut self.frames[reg],
            resampler: Resampler::default(),
        }
    }

    /// Extend the audio buffer with all of the audio from a stream.
    ///
    /// # Notes
    /// Do not call repeatedly on a stream with a differnt sample rate.  It will
    /// create extra "partial" samples.
    ///
    /// # Panics
    /// When an infinite stream is passed in.
    pub fn extend<G: Frame, M: Stream<G>>(&mut self, stream: M)
    where
        F::Chan: From<G::Chan>,
    {
        let mut temp_move = Self::with_frames::<[F; 0], f64>(0.0, []);
        swap(self, &mut temp_move);
        *self = temp_move.extend_internal(stream);
    }

    fn extend_internal<G: Frame, M: Stream<G>>(self, stream: M) -> Self
    where
        F::Chan: From<G::Chan>,
    {
        let s_rate = self.s_rate;

        // Get stream length.
        let srclen = stream
            .len()
            .expect("Audio::extend() called on infinite stream.");

        // Get source stream sample rate.
        let dstlen = if let Some(src_sr) = stream.sample_rate() {
            math::ceil(s_rate * srclen as f64 / src_sr) as usize
        } else {
            srclen
        };

        // Resize the audio buffer.
        let audio: Box<[F]> = self.into();
        let mut audio: Vec<F> = audio.into();
        let orig_len = audio.len();
        audio.resize_with(orig_len + dstlen, Default::default);
        let mut audio = Self::with_frames(s_rate, audio);

        // Write to new audio.
        let mut sink = audio.sink(orig_len..);
        sink.stream(stream);
        // Flush partial sample
        sink.flush();
        // Return audio
        audio
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

impl<'a, F: Frame> IntoIterator for &'a Audio<F> {
    type IntoIter = Cloned<Iter<'a, F>>;
    type Item = F;

    fn into_iter(self) -> Cloned<Iter<'a, F>> {
        self.frames.iter().cloned()
    }
}

impl<F: Frame> Stream<F> for &Audio<F> {
    fn sample_rate(&self) -> Option<f64> {
        Some(self.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.frames.len())
    }
}

/// A `Sink` created with `Audio.sink()`
struct AudioSink<'a, F: Frame> {
    s_rate: f64,
    frames: &'a mut [F],
    resampler: Resampler<F>,
}

impl<F: Frame> Sink<F> for AudioSink<'_, F> {
    fn sample_rate(&self) -> f64 {
        self.s_rate
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.resampler
    }

    fn buffer(&mut self) -> &mut [F] {
        self.frames
    }
}

/// A `Stream` created with `Audio.stream()`
struct AudioStream<
    'a,
    F: Frame,
    R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>,
> {
    audio: &'a Audio<F>,
    cursor: usize,
    range: R,
    size: usize,
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> Iterator
    for AudioStream<'_, F, R>
{
    type Item = F;

    fn next(&mut self) -> Option<F> {
        if !self.range.contains(&self.cursor) {
            return None;
        }
        let sample = self.audio.get(self.cursor)?;
        self.cursor += 1;
        Some(sample)
    }
}

impl<F: Frame, R: RangeBounds<usize> + SliceIndex<[F], Output = [F]>> Stream<F>
    for AudioStream<'_, F, R>
{
    fn sample_rate(&self) -> Option<f64> {
        Some(self.audio.sample_rate())
    }

    fn len(&self) -> Option<usize> {
        Some(self.size)
    }
}

/// A `Stream` created with `Audio.drain()`
struct AudioDrain<
    'a,
    F: Frame,
    R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone,
> {
    cursor: usize,
    audio: Vec<F>,
    region: R,
    buffer: &'a mut Audio<F>,
}

impl<F: Frame, R> Iterator for AudioDrain<'_, F, R>
where
    R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone,
{
    type Item = F;

    fn next(&mut self) -> Option<F> {
        if !self.region.contains(&self.cursor) {
            return None;
        }

        Some(self.audio[self.cursor])
    }
}

impl<'a, F: Frame, R> Stream<F> for AudioDrain<'_, F, R>
where
    R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone,
{
    fn sample_rate(&self) -> Option<f64> {
        Some(self.buffer.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.audio[self.region.clone()].len())
    }
}

impl<'a, F: Frame, R> Drop for AudioDrain<'_, F, R>
where
    R: RangeBounds<usize> + SliceIndex<[F], Output = [F]> + Clone,
{
    fn drop(&mut self) {
        {
            self.audio.drain(self.region.clone());
        }
        let mut buffer = Vec::new();
        swap(&mut buffer, &mut self.audio);
        *self.buffer = Audio::with_frames(self.buffer.s_rate, buffer);
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
        let capacity = frames.len() * size_of::<F>();
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
        let capacity = frames.len() * size_of::<F>() / 2;
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
        let capacity = frames.len() * size_of::<F>() / 4;
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
        let capacity = frames.len() * size_of::<F>() / 8;
        let slice = Box::<[F]>::into_raw(frames);
        let buffer: Box<[f64]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
