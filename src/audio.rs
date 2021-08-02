// Copyright © 2020-2021 The Fon Contributors.
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
    Sink, Stream,
};
use alloc::{
    boxed::Box,
    slice::{Iter, IterMut},
    vec,
    vec::Vec,
};
use core::borrow::BorrowMut;
use core::{fmt::Debug, mem::size_of, slice::from_raw_parts_mut};
use core::num::NonZeroU32;
use core::convert::TryInto;

/// Audio buffer (fixed-size array of audio [`Frame`](crate::frame::Frame)s at
/// sample rate specified in hertz).
///
/// `Audio` implements the [`Stream`](crate::Stream) trait.
#[derive(Debug)]
pub struct Audio<Chan: Channel, const CH: usize> {
    // Sample rate of the audio in hertz.
    sample_rate: NonZeroU32,
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
        Self::with_frames(hz, vec![Frame::<Chan, CH>::default(); len])
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
            sample_rate: hz.try_into().unwrap(),
            frames: frames.into(),
        }
    }

    /// Construct an `Audio` buffer from another `Audio` buffer of a differnt
    /// format.
    #[inline(always)]
    pub fn with_audio<Ch>(hz: u32, audio: &Audio<Ch, CH>) -> Self
    where
        Ch: Channel,
        Ch32: From<Ch>,
    {
        let rate = audio.len() as f64 * hz as f64 / audio.sample_rate().get() as f64;
        let mut output = Self::with_silence(hz, rate.ceil() as usize);
        let mut stream = Stream::new(hz);
        let mut sink = output.sink();
        stream.pipe(audio, &mut sink);
        stream.flush(&mut sink);
        output
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
    pub fn sample_rate(&self) -> NonZeroU32 {
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

    /// Sink audio into this audio buffer from a `Stream`.
    #[inline(always)]
    pub fn sink(&mut self) -> AudioSink<'_, Chan, CH> {
        AudioSink {
            index: 0,
            audio: self,
        }
    }
}

/// Returned from [`Audio::sink()`](crate::Audio::sink).
#[derive(Debug)]
pub struct AudioSink<'a, Chan: Channel, const CH: usize> {
    index: usize,
    audio: &'a mut Audio<Chan, CH>,
}

// Using '_ results in reserved lifetime error.
#[allow(single_use_lifetimes)]
impl<'a, T, Chan: Channel, const CH: usize> Sink<Chan, CH> for T
where
    T: BorrowMut<AudioSink<'a, Chan, CH>>,
{
    #[inline(always)]
    fn sample_rate(&self) -> NonZeroU32 {
        self.borrow().audio.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.borrow().audio.len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<Chan, CH>>) {
        let audio = self.borrow_mut();
        for frame in audio.audio.iter_mut().skip(audio.index) {
            *frame = if let Some(frame) = iter.next() {
                frame
            } else {
                break;
            };
            audio.index += 1;
        }
    }
}

/*impl<T, Chan: Channel, const CH: usize> Sink<Chan, CH> for T
    where T: BorrowMut<Audio<Chan, CH>>
{
    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.borrow().sample_rate
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.borrow().len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<Chan, CH>>) {
        for frame in self.borrow_mut().iter_mut() {
            *frame = iter.next().unwrap_or_default();
        }
    }
}*/

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
