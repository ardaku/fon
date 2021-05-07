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
pub struct Audio<Chan: Channel, const CH: usize, const HZ: u32>

(VecDeque<Frame<Chan, CH>>)
where
    Frame<Chan, CH>: Ops<Chan>
;

impl<Chan: Channel, const CH: usize, const HZ: u32> Audio<Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get an audio frame.
    pub fn get(&self, index: usize) -> Option<Frame<Chan, CH>> {
        self.0.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Frame<Chan, CH>> {
        self.0.get_mut(index)
    }

    /// Get a contiguous slice of all audio frames.  This may have to re-arrange
    /// memory if `drain()` was used, and could be slow.  If `drain()` was not
    /// called, this method should run in constant time.
    pub fn as_slice(&mut self) -> &mut [Frame<Chan, CH>] {
        self.0.make_contiguous()
    }

    /// Returns an iterator over the audio frames.
    pub fn iter(&self) -> Iter<'_, Frame<Chan, CH>> {
        self.0.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    pub fn iter_mut(&mut self) -> IterMut<'_, Frame<Chan, CH>> {
        self.0.iter_mut()
    }

    /// Construct an `Audio` buffer with all audio frames set to one value.
    pub fn with_frame(len: usize, frame: Frame<Chan, CH>) -> Self {
        Audio(vec![frame; len].into())
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    pub fn with_silence(len: usize) -> Self {
        Self::with_frame(len, Frame::<Chan, CH>::default())
    }

    /// Construct an [`Audio`](crate::Audio) buffer from the contents of a
    /// [`Stream`](crate::Stream).
    ///
    /// The audio format can be converted with this function.
    ///
    /// # Panics
    /// When an infinite stream is passed in.
    pub fn with_stream<C, M, const N: usize>(src: M) -> Self
    where
        Chan: From<C>,
        M: Stream<C, N>,
        C: Channel,
        Frame<C, N>: Ops<C>,
    {
        let mut audio = Self::with_frames::<[Frame<Chan, CH>; 0]>([]);
        audio.extend(src);
        audio
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_frames<B: Into<Box<[Frame<Chan, CH>]>>>(
        frames: B,
    ) -> Self {
        let frames: Vec<Frame<Chan, CH>> = frames.into().into();
        Audio(frames.into())
    }

    /// Get the length of the `Audio` buffer.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if `Audio` buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        AudioSink::<'_, _, CH, HZ> {
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
        let mut temp_move = Self::with_frames::<[Frame<Chan, CH>; 0]>([]);
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
        // Get stream length.
        let srclen = stream
            .len()
            .expect("Audio::extend() called on infinite stream.");

        // Get source stream sample rate.
        let dstlen = if let Some(src_sr) = stream.sample_rate() {
            (HZ as f32 * srclen as f32 / src_sr as f32).ceil() as usize
        } else {
            srclen
        };

        // Resize the audio buffer.
        let audio: Box<[Frame<Chan, CH>]> = self.into();
        let mut audio: Vec<Frame<Chan, CH>> = audio.into();
        let orig_len = audio.len();
        audio.resize_with(orig_len + dstlen, Default::default);
        let mut audio = Self::with_frames(audio);

        // Write to new audio.
        let mut sink = audio.sink(orig_len..);
        sink.stream(stream);
        // Flush partial sample
        sink.flush();
        // Return audio
        audio
    }
}

impl<const CH: usize, const HZ: u32> Audio<Ch16, CH, HZ>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(buffer: B) -> Self
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
        Audio(frames.into())
    }
}

impl<const CH: usize, const HZ: u32> Audio<Ch32, CH, HZ>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(buffer: B) -> Self
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
        Audio(frames.into())
    }
}

impl<const CH: usize, const HZ: u32> Audio<Ch16, CH, HZ>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Get view of samples as an `i16` slice.
    pub fn as_i16_slice(&mut self) -> &mut [i16] {
        let frames = self.0.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize, const HZ: u32> Audio<Ch32, CH, HZ>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Get view of samples as an `f32` slice.
    pub fn as_f32_slice(&mut self) -> &mut [f32] {
        let frames = self.0.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f32>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<'a, Chan, const CH: usize, const HZ: u32> IntoIterator for &'a Audio<Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>, Chan: Channel
{
    type IntoIter = Cloned<Iter<'a, Frame<Chan, CH>>>;
    type Item = Frame<Chan, CH>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}

impl<Chan, const CH: usize, const HZ: u32> Stream<Chan, CH> for &Audio<Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>, Chan: Channel
{
    fn sample_rate(&self) -> Option<u32> {
        Some(HZ)
    }

    fn len(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

/// A `Sink` created with `Audio.sink()`
struct AudioSink<'a, Chan: Channel, const CH: usize, const HZ: u32>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    frames: &'a mut [Frame<Chan, CH>],
    resampler: Resampler<Chan, CH>,
}

impl<Chan: Channel, const CH: usize, const HZ: u32> Sink<Chan, CH> for AudioSink<'_, Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn sample_rate(&self) -> u32 {
        HZ
    }

    fn resampler(&mut self) -> &mut Resampler<Chan, CH> {
        &mut self.resampler
    }

    fn buffer(&mut self) -> &mut [Frame<Chan, CH>] {
        self.frames
    }
}

/// A `Stream` created with `Audio.stream()`
struct AudioStream<'a, Chan, R, const CH: usize, const HZ: u32>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    R: RangeBounds<usize>
        + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
{
    audio: &'a Audio<Chan, CH, HZ>,
    cursor: usize,
    range: R,
    size: usize,
}

impl<Chan, R, const CH: usize, const HZ: u32> Iterator for AudioStream<'_, Chan, R, CH, HZ>
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

impl<Chan, R, const CH: usize, const HZ: u32> Stream<Chan, CH> for AudioStream<'_, Chan, R, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    R: RangeBounds<usize>
        + SliceIndex<[Frame<Chan, CH>], Output = [Frame<Chan, CH>]>,
{
    fn sample_rate(&self) -> Option<u32> {
        Some(HZ)
    }

    fn len(&self) -> Option<usize> {
        Some(self.size)
    }
}

/// A `Stream` created with `Audio.drain()`
struct AudioDrain<'a, Chan: Channel, const CH: usize, const HZ: u32>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    cursor: usize,
    buffer: &'a mut Audio<Chan, CH, HZ>,
}

impl<Chan: Channel, const CH: usize, const HZ: u32> Iterator for AudioDrain<'_, Chan, CH, HZ>
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

impl<'a, Chan: Channel, const CH: usize, const HZ: u32> Stream<Chan, CH>
    for AudioDrain<'_, Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn sample_rate(&self) -> Option<u32> {
        Some(HZ)
    }

    fn len(&self) -> Option<usize> {
        Some(self.buffer.len())
    }
}

impl<'a, Chan: Channel, const CH: usize, const HZ: u32> Drop for AudioDrain<'_, Chan, CH, HZ>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    fn drop(&mut self) {
        self.buffer.0.drain(..self.cursor);
    }
}

impl<Chan, const CH: usize, const HZ: u32> From<Audio<Chan, CH, HZ>> for Vec<Frame<Chan, CH>>
where
    Chan: Channel,
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH, HZ>) -> Self {
        audio.0.into()
    }
}

impl<Chan: Channel, const CH: usize, const HZ: u32> From<Audio<Chan, CH, HZ>>
    for Box<[Frame<Chan, CH>]>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH, HZ>) -> Self {
        let audio: Vec<Frame<Chan, CH>> = audio.0.into();
        audio.into()
    }
}

impl<const CH: usize, const HZ: u32> From<Audio<Ch16, CH, HZ>> for Box<[i16]>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch16, CH, HZ>) -> Self {
        let mut frames: Vec<Frame<Ch16, CH>> = audio.0.into();
        let capacity = frames.len() * size_of::<Frame<Ch16, CH>>() / 2;
        let buffer: Box<[i16]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize, const HZ: u32> From<Audio<Ch32, CH, HZ>> for Box<[f32]>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Get internal sample data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch32, CH, HZ>) -> Self {
        let mut frames: Vec<Frame<Ch32, CH>> = audio.0.into();
        let capacity = frames.len() * size_of::<Frame<Ch32, CH>>() / 4;
        let buffer: Box<[f32]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
