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
    chan::{Ch16, Ch24, Ch32, Ch64, Channel},
    frame::Frame,
    ops::Ops,
    Resampler, Sink, Stream,
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

/// Audio buffer (array of audio [`Frame`](crate::frame::Frame)s at sample rate
/// specified in hertz).
///
/// `Audio` implements the [`Stream`](crate::Stream) trait.
#[derive(Debug)]
pub struct Audio<Chan: Channel, const CH: usize>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    s_rate: u32,
    frames: VecDeque<Frame<Chan, CH>>,
}

impl<Chan: Channel, const CH: usize> Audio<Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get an audio frame.
    pub fn get(&self, index: usize) -> Option<Frame<Chan, CH>> {
        self.frames.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Frame<Chan, CH>> {
        self.frames.get_mut(index)
    }

    /// Get a contiguous slice of all audio frames.  This may have to re-arrange
    /// memory if `drain()` was used, and could be slow.  If `drain()` was not
    /// called, this method should run in constant time.
    pub fn as_slice(&mut self) -> &mut [Frame<Chan, CH>] {
        self.frames.make_contiguous()
    }

    /// Returns an iterator over the audio frames.
    pub fn iter(&self) -> Iter<'_, Frame<Chan, CH>> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    pub fn iter_mut(&mut self) -> IterMut<'_, Frame<Chan, CH>> {
        self.frames.iter_mut()
    }

    /// Construct an `Audio` buffer with all audio frames set to one value.
    pub fn with_frame(s_rate: u32, len: usize, frame: Frame<Chan, CH>) -> Self {
        let frames = vec![frame; len].into();
        Audio { s_rate, frames }
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    pub fn with_silence(s_rate: u32, len: usize) -> Self {
        Self::with_frame(s_rate, len, Frame::<Chan, CH>::default())
    }

    /// Construct an [`Audio`](crate::Audio) buffer from the contents of a
    /// [`Stream`](crate::Stream).
    ///
    /// The audio format can be converted with this function.
    ///
    /// # Panics
    /// When an infinite stream is passed in.
    pub fn with_stream<C, M, const N: usize>(s_rate: u32, src: M) -> Self
    where
        Chan: From<C>,
        M: Stream<C, N>,
        C: Channel,
        Frame<C, N>: Ops<C>,
    {
        let mut audio = Self::with_frames::<[Frame<Chan, CH>; 0]>(s_rate, []);
        audio.extend(src);
        audio
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_frames<B: Into<Box<[Frame<Chan, CH>]>>>(
        s_rate: u32,
        frames: B,
    ) -> Self {
        let frames: Vec<Frame<Chan, CH>> = frames.into().into();
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
        R: 'a
            + RangeBounds<usize>
            + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
    >(
        &'a mut self,
        reg: R,
    ) -> impl Sink<Chan, CH> + '_ {
        AudioSink {
            s_rate: self.sample_rate(),
            frames: &mut self.as_slice()[reg],
            resampler: Resampler::default(),
        }
    }

    /// Create a draining audio stream from this `Audio` buffer.  When the
    /// stream is dropped, only sinked audio samples will be removed.
    pub fn drain(&mut self) -> impl Stream<Chan, CH> + '_ {
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
    pub fn extend<C: Channel, M: Stream<C, N>, const N: usize>(
        &mut self,
        stream: M,
    ) where
        Chan: From<C>,
        Frame<C, N>: Ops<C>,
    {
        let mut temp_move = Self::with_frames::<[Frame<Chan, CH>; 0]>(0, []);
        swap(self, &mut temp_move);
        *self = temp_move.extend_internal(stream);
    }

    fn extend_internal<C: Channel, M: Stream<C, N>, const N: usize>(
        self,
        stream: M,
    ) -> Self
    where
        Chan: From<C>,
        Frame<C, N>: Ops<C>,
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
        let audio: Box<[Frame<Chan, CH>]> = self.into();
        let mut audio: Vec<Frame<Chan, CH>> = audio.into();
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

impl<const CH: usize> Audio<Ch16, CH>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
    {
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<Frame<Ch16, CH>>();
        assert_eq!(0, bytes % size_of::<Frame<Ch16, CH>>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let frames: Box<[Frame<Ch16, CH>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Ch16, CH>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Ch16, CH>> = frames.into();
        Audio {
            s_rate,
            frames: frames.into(),
        }
    }
}

impl<const CH: usize> Audio<Ch32, CH>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
    {
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * size_of::<f32>();
        let len = bytes / size_of::<Frame<Ch32, CH>>();
        assert_eq!(0, bytes % size_of::<Frame<Ch32, CH>>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let frames: Box<[Frame<Ch32, CH>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Ch32, CH>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Ch32, CH>> = frames.into();
        Audio {
            s_rate,
            frames: frames.into(),
        }
    }
}

impl<const CH: usize> Audio<Ch16, CH>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
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

impl<const CH: usize> Audio<Ch32, CH>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
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

impl<'a, Chan: Channel, const CH: usize> IntoIterator for &'a Audio<Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    type IntoIter = Cloned<Iter<'a, Frame<Chan, CH>>>;
    type Item = Frame<Chan, CH>;

    fn into_iter(self) -> Self::IntoIter {
        self.frames.iter().cloned()
    }
}

impl<Chan: Channel, const CH: usize> Stream<Chan, CH> for &Audio<Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn sample_rate(&self) -> Option<u32> {
        Some(self.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.frames.len())
    }
}

/// A `Sink` created with `Audio.sink()`
struct AudioSink<'a, Chan: Channel, const CH: usize>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    s_rate: u32,
    frames: &'a mut [Frame<Chan, CH>],
    resampler: Resampler<Chan, CH>,
}

impl<Chan: Channel, const CH: usize> Sink<Chan, CH> for AudioSink<'_, Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn sample_rate(&self) -> u32 {
        self.s_rate
    }

    fn resampler(&mut self) -> &mut Resampler<Chan, CH> {
        &mut self.resampler
    }

    fn buffer(&mut self) -> &mut [Frame<Chan, CH>] {
        self.frames
    }
}

/// A `Stream` created with `Audio.stream()`
struct AudioStream<'a, Chan, R, const CH: usize>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    R: RangeBounds<usize>
        + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
{
    audio: &'a Audio<Chan, CH>,
    cursor: usize,
    range: R,
    size: usize,
}

impl<Chan, R, const CH: usize> Iterator for AudioStream<'_, Chan, R, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    R: RangeBounds<usize>
        + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
{
    type Item = Frame<Chan, CH>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.range.contains(&self.cursor) {
            return None;
        }
        let sample = self.audio.get(self.cursor)?;
        self.cursor += 1;
        Some(sample)
    }
}

impl<Chan, R, const CH: usize> Stream<Chan, CH> for AudioStream<'_, Chan, R, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    R: RangeBounds<usize>
        + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
{
    fn sample_rate(&self) -> Option<u32> {
        Some(self.audio.sample_rate())
    }

    fn len(&self) -> Option<usize> {
        Some(self.size)
    }
}

/// A `Stream` created with `Audio.drain()`
struct AudioDrain<'a, Chan: Channel, const CH: usize>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    cursor: usize,
    buffer: &'a mut Audio<Chan, CH>,
}

impl<Chan: Channel, const CH: usize> Iterator for AudioDrain<'_, Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    type Item = Frame<Chan, CH>;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.buffer.get(self.cursor)?;
        self.cursor += 1;
        Some(sample)
    }
}

impl<'a, Chan: Channel, const CH: usize> Stream<Chan, CH>
    for AudioDrain<'_, Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn sample_rate(&self) -> Option<u32> {
        Some(self.buffer.s_rate)
    }

    fn len(&self) -> Option<usize> {
        Some(self.buffer.len())
    }
}

impl<'a, Chan: Channel, const CH: usize> Drop for AudioDrain<'_, Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn drop(&mut self) {
        self.buffer.frames.drain(..self.cursor);
    }
}

impl<Chan, const CH: usize> From<Audio<Chan, CH>> for Vec<Frame<Chan, CH>>
where
    Chan: Channel,
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        audio.frames.into()
    }
}

impl<Chan: Channel, const CH: usize> From<Audio<Chan, CH>>
    for Box<[Frame<Chan, CH>]>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        let audio: Vec<Frame<Chan, CH>> = audio.frames.into();
        audio.into()
    }
}

impl<const CH: usize> From<Audio<Ch16, CH>> for Box<[i16]>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch16, CH>) -> Self {
        let mut frames: Vec<Frame<Ch16, CH>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Ch16, CH>>() / 2;
        let buffer: Box<[i16]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize> From<Audio<Ch32, CH>> for Box<[f32]>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch32, CH>) -> Self {
        let mut frames: Vec<Frame<Ch32, CH>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Ch32, CH>>() / 4;
        let buffer: Box<[f32]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
