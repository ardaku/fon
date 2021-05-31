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
    Stream,
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
use core::{fmt::Debug, iter::Cloned, mem::size_of, slice::from_raw_parts_mut};

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
    // Sample rate of the audio in hertz.
    rate: u32,
    // Audio sample data
    // FIXME: pub
    pub(crate) data: VecDeque<Frame<Chan, CH>>,
}

impl<Chan: Channel, const CH: usize> Audio<Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Construct an empty `Audio` buffer.
    #[inline(always)]
    pub fn new(hz: u32) -> Self {
        Self::with_frames::<[Frame<Chan, CH>; 0]>(hz, [])
    }

    /// Construct an `Audio` buffer with all all samples set to zero.
    #[inline(always)]
    pub fn with_silence(hz: u32, len: usize) -> Self {
        Self::with_frame(hz, Frame::<Chan, CH>::default(), len)
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    #[inline(always)]
    pub fn with_frames<B>(hz: u32, frames: B) -> Self
    where
        B: Into<Box<[Frame<Chan, CH>]>>,
    {
        Audio {
            rate: hz,
            data: Vec::<Frame<Chan, CH>>::from(frames.into()).into(),
        }
    }

    /// Construct an `Audio` buffer with all audio frames set to one value.
    #[inline(always)]
    pub fn with_frame(hz: u32, frame: Frame<Chan, CH>, len: usize) -> Self {
        Self::with_frames(hz, vec![frame; len])
    }

    /// Construct an [`Audio`](crate::Audio) buffer from the contents of a
    /// [`Stream`](crate::Stream).
    ///
    /// The audio format can be converted with this function.  Sample rate in
    /// hertz is taken from the source (`src`) stream.
    #[inline(always)]
    pub fn with_stream<M, C: Channel, const N: usize>(
        mut src: M,
        len: usize,
    ) -> Self
    where
        M: Stream<C, N>,
        Frame<C, N>: Ops<C>,
        Frame<Chan, N>: Ops<Chan>,
        Chan: From<C>,
    {
        let mut audio = Self::new(src.sample_rate().unwrap());
        src.extend::<Chan, CH>(&mut audio, len);
        audio
    }

    /// Clear the audio buffer.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.data.clear()
    }

    /// Get an audio frame.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<Frame<Chan, CH>> {
        self.data.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Frame<Chan, CH>> {
        self.data.get_mut(index)
    }

    /// Get a contiguous slice of all audio frames.  This may have to re-arrange
    /// memory if `drain()` was used, and could be slow.  If `drain()` was not
    /// called, this method should run in constant time.
    #[inline(always)]
    pub fn as_slice(&mut self) -> &mut [Frame<Chan, CH>] {
        self.data.make_contiguous()
    }

    /// Returns an iterator over the audio frames.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Frame<Chan, CH>> {
        self.data.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Frame<Chan, CH>> {
        self.data.iter_mut()
    }

    /// Get the length of the `Audio` buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if `Audio` buffer is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a draining audio stream from this `Audio` buffer.  When the
    /// stream is dropped, only sinked audio samples will be removed.
    #[inline(always)]
    pub fn drain(&mut self) -> impl Stream<Chan, CH> + '_ {
        AudioDrain {
            cursor: 0,
            buffer: self,
        }
    }
}

impl<const CH: usize> Audio<Ch16, CH>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(hz: u32, buffer: B) -> Self
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
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `i16` slice.
    pub fn as_i16_slice(&mut self) -> &mut [i16] {
        let frames = self.data.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize> Audio<Ch24, CH>
where
    Frame<Ch24, CH>: Ops<Ch24>,
{
    /// Construct an `Audio` buffer from an `u8` buffer.
    #[allow(unsafe_code)]
    pub fn with_u8_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[u8]>>,
    {
        let buffer: Box<[u8]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<Frame<Ch16, CH>>();
        assert_eq!(0, bytes % size_of::<Frame<Ch16, CH>>());
        let slice = Box::<[u8]>::into_raw(buffer);
        let frames: Box<[Frame<Ch24, CH>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Ch24, CH>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Ch24, CH>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `u8` slice.
    pub fn as_u8_slice(&mut self) -> &mut [u8] {
        let frames = self.data.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<u8>();
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
    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(hz: u32, buffer: B) -> Self
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
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `f32` slice.
    pub fn as_f32_slice(&mut self) -> &mut [f32] {
        let frames = self.data.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f32>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize> Audio<Ch64, CH>
where
    Frame<Ch64, CH>: Ops<Ch64>,
{
    /// Construct an `Audio` buffer from an `f64` buffer.
    #[allow(unsafe_code)]
    pub fn with_f64_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[f64]>>,
    {
        let buffer: Box<[f64]> = buffer.into();
        let bytes = buffer.len() * size_of::<f64>();
        let len = bytes / size_of::<Frame<Ch64, CH>>();
        assert_eq!(0, bytes % size_of::<Frame<Ch64, CH>>());
        let slice = Box::<[f64]>::into_raw(buffer);
        let frames: Box<[Frame<Ch64, CH>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Ch64, CH>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Ch64, CH>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `f64` slice.
    pub fn as_f64_slice(&mut self) -> &mut [f64] {
        let frames = self.data.make_contiguous();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f64>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<'a, Chan, const CH: usize> IntoIterator for &'a Audio<Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
{
    type IntoIter = Cloned<Iter<'a, Frame<Chan, CH>>>;
    type Item = Frame<Chan, CH>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().cloned()
    }
}

impl<Chan, F, const CH: usize> Stream<Chan, CH> for F
where
    Frame<Chan, CH>: Ops<Chan>,
    Chan: Channel,
    F: core::borrow::Borrow<Audio<Chan, CH>>,
{
    #[inline(always)]
    fn sample_rate(&self) -> Option<u32> {
        Some(self.borrow().rate)
    }

    #[inline(always)]
    fn extend<C: Channel, const N: usize>(
        &mut self,
        buffer: &mut Audio<C, N>,
        len: usize,
    ) where
        C: From<Chan>,
        Frame<C, N>: Ops<C>,
    {
        let this = self.borrow();
        let zeros = if len > this.len() {
            len - this.len()
        } else {
            0
        };
        buffer
            .data
            .extend(this.into_iter().map(|x| x.to()).take(len));
        Frame::<C, N>::default().extend(buffer, zeros);
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
    #[inline(always)]
    fn sample_rate(&self) -> Option<u32> {
        Some(self.buffer.rate)
    }

    #[inline(always)]
    fn extend<C: Channel, const N: usize>(
        &mut self,
        buffer: &mut Audio<C, N>,
        len: usize,
    ) where
        C: From<Chan>,
        Frame<C, N>: Ops<C>,
        Frame<C, CH>: Ops<C>,
    {
        (*self.buffer).extend(buffer, len);
    }
}

impl<'a, Chan: Channel, const CH: usize> Drop for AudioDrain<'_, Chan, CH>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    #[inline(always)]
    fn drop(&mut self) {
        self.buffer.data.drain(..self.cursor);
    }
}

impl<Chan, const CH: usize> From<Audio<Chan, CH>> for Vec<Frame<Chan, CH>>
where
    Chan: Channel,
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        audio.data.into()
    }
}

impl<Chan: Channel, const CH: usize> From<Audio<Chan, CH>>
    for Box<[Frame<Chan, CH>]>
where
    Frame<Chan, CH>: Ops<Chan>,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        let audio: Vec<Frame<Chan, CH>> = audio.data.into();
        audio.into()
    }
}

impl<const CH: usize> From<Audio<Ch16, CH>> for Box<[i16]>
where
    Frame<Ch16, CH>: Ops<Ch16>,
{
    /// Get internal sample data as boxed slice of *i16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch16, CH>) -> Self {
        let mut frames: Vec<Frame<Ch16, CH>> = audio.data.into();
        let capacity = frames.len() * size_of::<Frame<Ch16, CH>>() / 2;
        let buffer: Box<[i16]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize> From<Audio<Ch24, CH>> for Box<[u8]>
where
    Frame<Ch24, CH>: Ops<Ch24>,
{
    /// Get internal sample data as boxed slice of *u8*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch24, CH>) -> Self {
        let mut frames: Vec<Frame<Ch24, CH>> = audio.data.into();
        let capacity = frames.len() * size_of::<Frame<Ch24, CH>>() / 3;
        let buffer: Box<[u8]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut u8;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize> From<Audio<Ch32, CH>> for Box<[f32]>
where
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Get internal sample data as boxed slice of *f32*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch32, CH>) -> Self {
        let mut frames: Vec<Frame<Ch32, CH>> = audio.data.into();
        let capacity = frames.len() * size_of::<Frame<Ch32, CH>>() / 4;
        let buffer: Box<[f32]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize> From<Audio<Ch64, CH>> for Box<[f64]>
where
    Frame<Ch64, CH>: Ops<Ch64>,
{
    /// Get internal sample data as boxed slice of *f64*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch64, CH>) -> Self {
        let mut frames: Vec<Frame<Ch64, CH>> = audio.data.into();
        let capacity = frames.len() * size_of::<Frame<Ch64, CH>>() / 8;
        let buffer: Box<[f64]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
