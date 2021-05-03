// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{
    chan::{Ch16, Ch24, Ch32},
    Frame, Resampler, Sink, Stream,
};
use alloc::{
    boxed::Box,
    collections::{
        vec_deque::{Iter, IterMut},
        VecDeque,
    },
    vec,
    vec::Vec,
};
use core::{
    fmt::Debug,
    iter::Cloned,
    mem::{size_of, swap},
    ops::RangeBounds,
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

/// Audio buffer (array of audio [`Frame`](crate::Frame)s at sample rate
/// specified in hertz).
///
/// `Audio` implements the [`Stream`](crate::Stream) trait.
#[derive(Debug)]
pub struct Audio<F: Frame> {
    s_rate: u32,
    frames: VecDeque<F>,
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

    /// Get a contiguous slice of all audio frames.  This may have to re-arrange
    /// memory if `drain()` was used, and could be slow.  If `drain()` was not
    /// called, this method should run in constant time.
    pub fn as_slice(&mut self) -> &mut [F] {
        self.frames.make_contiguous()
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
    pub fn with_frame(s_rate: u32, len: usize, frame: F) -> Self {
        let frames = vec![frame; len].into();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    pub fn with_silence(s_rate: u32, len: usize) -> Self {
        Self::with_frame(s_rate, len, F::default())
    }

    /// Construct an [`Audio`](crate::Audio) buffer from the contents of a
    /// [`Stream`](crate::Stream).
    ///
    /// The audio format can be converted with this function.
    ///
    /// # Panics
    /// When an infinite stream is passed in.
    pub fn with_stream<S, M>(s_rate: u32, src: M) -> Self
    where
        F::Chan: From<S::Chan>,
        M: Stream<S>,
        S: Frame,
    {
        let mut audio = Self::with_frames::<[F; 0]>(s_rate, []);
        audio.extend(src);
        audio
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_frames<B: Into<Box<[F]>>>(
        s_rate: u32,
        frames: B,
    ) -> Self {
        let frames: Vec<F> = frames.into().into();
        Audio {
            s_rate,
            frames: frames.into(),
        }
    }

    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
        F: Frame<Chan = Ch16>,
    {
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<F>();
        assert_eq!(0, bytes % size_of::<F>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<F> = frames.into();
        Audio {
            s_rate,
            frames: frames.into(),
        }
    }

    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
        F: Frame<Chan = Ch32>,
    {
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * size_of::<f32>();
        let len = bytes / size_of::<F>();
        assert_eq!(0, bytes % size_of::<F>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let frames: Box<[F]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut F;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<F> = frames.into();
        Audio {
            s_rate,
            frames: frames.into(),
        }
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

    /// Create an audio sink to overwrite a region of this `Audio` buffer.
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
        AudioSink {
            s_rate: self.sample_rate(),
            frames: &mut self.as_slice()[reg],
            resampler: Resampler::default(),
        }
    }

    /// Create a draining audio stream from this `Audio` buffer.  When the
    /// stream is dropped, only sinked audio samples will be removed.
    pub fn drain(&mut self) -> impl Stream<F> + '_ {
        AudioDrain {
            cursor: 0,
            buffer: self,
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
        let mut temp_move = Self::with_frames::<[F; 0]>(0, []);
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
            (s_rate as f32 * srclen as f32 / src_sr as f32).ceil() as usize
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

impl<F: Frame<Chan = Ch16>> Audio<F> {
    /// Get view of samples as an `i16` slice.
    pub fn as_i16_slice(&mut self) -> &mut [i16] {
        let frames = self.frames.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<S: Frame<Chan = Ch32>> Audio<S> {
    /// Get view of samples as an `f32` slice.
    pub fn as_f32_slice(&mut self) -> &mut [f32] {
        let frames = self.frames.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f32>();
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
    fn sample_rate(&self) -> Option<u32> {
        Some(self.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.frames.len())
    }
}

/// A `Sink` created with `Audio.sink()`
struct AudioSink<'a, F: Frame> {
    s_rate: u32,
    frames: &'a mut [F],
    resampler: Resampler<F>,
}

impl<F: Frame> Sink<F> for AudioSink<'_, F> {
    fn sample_rate(&self) -> u32 {
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
// FIXME
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
    fn sample_rate(&self) -> Option<u32> {
        Some(self.audio.sample_rate())
    }

    fn len(&self) -> Option<usize> {
        Some(self.size)
    }
}

/// A `Stream` created with `Audio.drain()`
struct AudioDrain<'a, F: Frame> {
    cursor: usize,
    buffer: &'a mut Audio<F>,
}

impl<F: Frame> Iterator for AudioDrain<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<F> {
        let sample = self.buffer.get(self.cursor)?;
        self.cursor += 1;
        Some(sample)
    }
}

impl<'a, F: Frame> Stream<F> for AudioDrain<'_, F> {
    fn sample_rate(&self) -> Option<u32> {
        Some(self.buffer.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.buffer.len())
    }
}

impl<'a, F: Frame> Drop for AudioDrain<'_, F> {
    fn drop(&mut self) {
        self.buffer.frames.drain(..self.cursor);
    }
}

impl<F: Frame> From<Audio<F>> for Vec<F> {
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<F>) -> Self {
        audio.frames.into()
    }
}

impl<F: Frame> From<Audio<F>> for Box<[F]> {
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<F>) -> Self {
        let audio: Vec<F> = audio.frames.into();
        audio.into()
    }
}

impl<F: Frame<Chan = Ch16>> From<Audio<F>> for Box<[i16]> {
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let mut frames: Vec<F> = audio.frames.into();
        let capacity = frames.len() * size_of::<F>() / 2;
        let buffer: Box<[i16]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<F: Frame<Chan = Ch32>> From<Audio<F>> for Box<[f32]> {
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<F>) -> Self {
        let mut frames: Vec<F> = audio.frames.into();
        let capacity = frames.len() * size_of::<F>() / 4;
        let buffer: Box<[f32]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
