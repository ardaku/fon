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
    Stream,
};
use alloc::{
    boxed::Box,
    slice::{Iter, IterMut},
    vec,
    vec::Vec,
};
use core::{fmt::Debug, mem::size_of, slice::from_raw_parts_mut};

/// Audio buffer (fixed-size array of audio [`Frame`](crate::frame::Frame)s at
/// sample rate specified in hertz).
///
/// `Audio` implements the [`Stream`](crate::Stream) trait.
#[derive(Debug)]
pub struct Audio<Chan: Channel, const CH: usize> {
    // Sample rate of the audio in hertz.
    sample_rate: u32,
    // Audio frames.
    frames: Box<[Frame<Chan, CH>]>,
}

impl<Chan: Channel, const CH: usize> Audio<Chan, CH> {
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
            sample_rate: hz,
            frames: frames.into(),
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
        src: M,
        len: usize,
    ) -> Self
    where
        M: Stream<C, N>,
        Chan: From<C>,
    {
        let mut audio = Self::with_silence(src.sample_rate(), len);
        audio.sink(src);
        audio
    }

    /// Get an audio frame.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<Frame<Chan, CH>> {
        self.frames.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Frame<Chan, CH>> {
        self.frames.get_mut(index)
    }

    /// Get a slice of all audio frames.
    #[inline(always)]
    pub fn as_slice(&self) -> &[Frame<Chan, CH>] {
        &*self.frames
    }

    /// Get a slice of all audio frames.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Frame<Chan, CH>] {
        &mut *self.frames
    }

    /// Returns an iterator over the audio frames.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Frame<Chan, CH>> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Frame<Chan, CH>> {
        self.frames.iter_mut()
    }

    /// Get the sample rate of this audio buffer.
    #[inline(always)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the length of the `Audio` buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Check if `Audio` buffer is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Silence the audio buffer.
    #[inline(always)]
    pub fn silence(&mut self) {
        for f in self.frames.iter_mut() {
            *f = Frame::<Chan, CH>::default()
        }
    }

    /// Sink samples into this buffer from a stream.
    #[inline(always)]
    pub fn sink<Ch, S, const N: usize>(&mut self, mut stream: S)
    where
        Ch: Channel,
        S: Stream<Ch, N>,
        Chan: From<Ch>,
    {
        stream.sink(self)
    }
}

impl<const CH: usize> Audio<Ch16, CH> {
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
        let frames = self.as_mut_slice();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<i16>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize> Audio<Ch24, CH> {
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
        let frames = self.as_mut_slice();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<u8>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize> Audio<Ch32, CH> {
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
        let frames = self.as_mut_slice();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f32>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<const CH: usize> Audio<Ch64, CH> {
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
        let frames = self.as_mut_slice();
        unsafe {
            let (prefix, v, suffix) = frames.align_to_mut::<f64>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<Chan, F, const CH: usize> Stream<Chan, CH> for F
where
    Chan: Channel,
    F: core::borrow::Borrow<Audio<Chan, CH>>,
{
    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.borrow().sample_rate
    }

    #[inline(always)]
    fn sink<C: Channel, const N: usize>(&mut self, buf: &mut Audio<C, N>)
    where
        C: From<Chan>,
    {
        assert_eq!(self.sample_rate(), buf.sample_rate());

        // Get iterator
        let mut it = self.borrow().iter().cloned().map(|x| x.to());
        // Convert channel type.
        for out in buf.iter_mut() {
            *out = it.next().unwrap_or_default();
        }
    }
}

impl<Chan, const CH: usize> From<Audio<Chan, CH>> for Vec<Frame<Chan, CH>>
where
    Chan: Channel,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        audio.frames.into()
    }
}

impl<Chan: Channel, const CH: usize> From<Audio<Chan, CH>>
    for Box<[Frame<Chan, CH>]>
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<Chan, CH>) -> Self {
        let audio: Vec<Frame<Chan, CH>> = audio.frames.into();
        audio.into()
    }
}

impl<const CH: usize> From<Audio<Ch16, CH>> for Box<[i16]> {
    /// Get internal sample data as boxed slice of *i16*.
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

impl<const CH: usize> From<Audio<Ch24, CH>> for Box<[u8]> {
    /// Get internal sample data as boxed slice of *u8*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch24, CH>) -> Self {
        let mut frames: Vec<Frame<Ch24, CH>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Ch24, CH>>() / 3;
        let buffer: Box<[u8]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut u8;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const CH: usize> From<Audio<Ch32, CH>> for Box<[f32]> {
    /// Get internal sample data as boxed slice of *f32*.
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

impl<const CH: usize> From<Audio<Ch64, CH>> for Box<[f64]> {
    /// Get internal sample data as boxed slice of *f64*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Ch64, CH>) -> Self {
        let mut frames: Vec<Frame<Ch64, CH>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Ch64, CH>>() / 8;
        let buffer: Box<[f64]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
