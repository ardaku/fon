use alloc::{
    boxed::Box,
    slice::{Iter, IterMut},
    vec,
    vec::Vec,
};
use core::{
    convert::TryInto, fmt::Debug, mem::size_of, num::NonZeroU32,
    slice::from_raw_parts_mut,
};

#[cfg(not(test))]
use crate::math::Libm;
use crate::{
    chan::{Channel, Samp16, Samp24, Samp32, Samp64},
    frame::Frame,
    Resampler, Sink,
};

/// Audio buffer (fixed-size array of audio [`Frame`](crate::frame::Frame)s at
/// sample rate specified in hertz).
#[derive(Debug)]
pub struct Audio<C: Channel, const COUNT: usize> {
    // Sample rate of the audio in hertz.
    sample_rate: NonZeroU32,
    // Audio frames.
    frames: Box<[Frame<C, COUNT>]>,
}

impl<C: Channel, const COUNT: usize> Audio<C, COUNT> {
    /// Construct an `Audio` buffer with all all samples set to zero.
    #[inline(always)]
    pub fn with_silence(hz: u32, len: usize) -> Self {
        Self::with_frames(hz, vec![Frame::<C, COUNT>::default(); len])
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the sample data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    #[inline(always)]
    pub fn with_frames<B>(hz: u32, frames: B) -> Self
    where
        B: Into<Box<[Frame<C, COUNT>]>>,
    {
        Audio {
            sample_rate: hz.try_into().unwrap(),
            frames: frames.into(),
        }
    }

    /// Construct an `Audio` buffer from another `Audio` buffer of a different
    /// format.
    #[inline(always)]
    pub fn with_audio<F, const N: usize>(hz: u32, audio: &Audio<F, N>) -> Self
    where
        F: Channel,
        Samp32: From<F>,
        C: From<F>,
    {
        let len =
            audio.len() as f64 * hz as f64 / audio.sample_rate().get() as f64;
        let mut output = Self::with_silence(hz, len.ceil() as usize);
        let mut stream = Resampler::new(hz);
        let mut sink = crate::SinkTo::<_, C, _, COUNT, N>::new(output.sink());
        stream.pipe(audio, &mut sink);
        stream.flush(&mut sink);
        output
    }

    /// Get an audio frame.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<Frame<C, COUNT>> {
        self.frames.get(index).cloned()
    }

    /// Get a mutable reference to an audio frame.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Frame<C, COUNT>> {
        self.frames.get_mut(index)
    }

    /// Get a slice of all audio frames.
    #[inline(always)]
    pub fn as_slice(&self) -> &[Frame<C, COUNT>] {
        &self.frames
    }

    /// Get a slice of all audio frames.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Frame<C, COUNT>] {
        &mut self.frames
    }

    /// Returns an iterator over the audio frames.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Frame<C, COUNT>> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying each audio frame.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Frame<C, COUNT>> {
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
            *f = Frame::<C, COUNT>::default()
        }
    }

    /// Sink audio into this audio buffer from a [`Resampler`].
    #[inline(always)]
    pub fn sink(&mut self) -> AudioSink<'_, C, COUNT> {
        AudioSink {
            index: 0,
            audio: self,
        }
    }
}

/// Returned from [`Audio::sink()`](crate::Audio::sink).
#[derive(Debug)]
pub struct AudioSink<'a, C: Channel, const COUNT: usize> {
    index: usize,
    audio: &'a mut Audio<C, COUNT>,
}

// Using '_ results in reserved lifetime error.
#[allow(single_use_lifetimes)]
impl<C: Channel, const COUNT: usize> Sink<C, COUNT>
    for AudioSink<'_, C, COUNT>
{
    #[inline(always)]
    fn sample_rate(&self) -> NonZeroU32 {
        self.audio.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.audio.len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<C, COUNT>>) {
        let mut this = self;
        Sink::<C, COUNT>::sink_with(&mut this, iter)
    }
}

impl<C: Channel, const COUNT: usize> Sink<C, COUNT>
    for &mut AudioSink<'_, C, COUNT>
{
    #[inline(always)]
    fn sample_rate(&self) -> NonZeroU32 {
        self.audio.sample_rate()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.audio.len()
    }

    #[inline(always)]
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<C, COUNT>>) {
        for frame in self.audio.iter_mut().skip(self.index) {
            *frame = if let Some(frame) = iter.next() {
                frame
            } else {
                break;
            };
            self.index += 1;
        }
    }
}

impl<const COUNT: usize> Audio<Samp16, COUNT> {
    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
    {
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<Frame<Samp16, COUNT>>();
        assert_eq!(0, bytes % size_of::<Frame<Samp16, COUNT>>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let frames: Box<[Frame<Samp16, COUNT>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Samp16, COUNT>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Samp16, COUNT>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `i16` slice.
    #[allow(unsafe_code)]
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

impl<const COUNT: usize> Audio<Samp24, COUNT> {
    /// Construct an `Audio` buffer from an `u8` buffer.
    #[allow(unsafe_code)]
    pub fn with_u8_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[u8]>>,
    {
        let buffer: Box<[u8]> = buffer.into();
        let bytes = buffer.len() * size_of::<i16>();
        let len = bytes / size_of::<Frame<Samp16, COUNT>>();
        assert_eq!(0, bytes % size_of::<Frame<Samp16, COUNT>>());
        let slice = Box::<[u8]>::into_raw(buffer);
        let frames: Box<[Frame<Samp24, COUNT>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Samp24, COUNT>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Samp24, COUNT>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `u8` slice.
    #[allow(unsafe_code)]
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

impl<const COUNT: usize> Audio<Samp32, COUNT> {
    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
    {
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * size_of::<f32>();
        let len = bytes / size_of::<Frame<Samp32, COUNT>>();
        assert_eq!(0, bytes % size_of::<Frame<Samp32, COUNT>>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let frames: Box<[Frame<Samp32, COUNT>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Samp32, COUNT>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Samp32, COUNT>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `f32` slice.
    #[allow(unsafe_code)]
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

impl<const COUNT: usize> Audio<Samp64, COUNT> {
    /// Construct an `Audio` buffer from an `f64` buffer.
    #[allow(unsafe_code)]
    pub fn with_f64_buffer<B>(hz: u32, buffer: B) -> Self
    where
        B: Into<Box<[f64]>>,
    {
        let buffer: Box<[f64]> = buffer.into();
        let bytes = buffer.len() * size_of::<f64>();
        let len = bytes / size_of::<Frame<Samp64, COUNT>>();
        assert_eq!(0, bytes % size_of::<Frame<Samp64, COUNT>>());
        let slice = Box::<[f64]>::into_raw(buffer);
        let frames: Box<[Frame<Samp64, COUNT>]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut Frame<Samp64, COUNT>;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        let frames: Vec<Frame<Samp64, COUNT>> = frames.into();
        Audio::with_frames(hz, frames)
    }

    /// Get view of samples as an `f64` slice.
    #[allow(unsafe_code)]
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

impl<C, const COUNT: usize> From<Audio<C, COUNT>> for Vec<Frame<C, COUNT>>
where
    C: Channel,
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<C, COUNT>) -> Self {
        audio.frames.into()
    }
}

impl<C: Channel, const COUNT: usize> From<Audio<C, COUNT>>
    for Box<[Frame<C, COUNT>]>
{
    /// Get internal sample data as `Vec` of audio frames.
    fn from(audio: Audio<C, COUNT>) -> Self {
        let audio: Vec<Frame<C, COUNT>> = audio.frames.into();
        audio.into()
    }
}

impl<const COUNT: usize> From<Audio<Samp16, COUNT>> for Box<[i16]> {
    /// Get internal sample data as boxed slice of *i16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Samp16, COUNT>) -> Self {
        let mut frames: Vec<Frame<Samp16, COUNT>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Samp16, COUNT>>() / 2;
        let buffer: Box<[i16]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const COUNT: usize> From<Audio<Samp24, COUNT>> for Box<[u8]> {
    /// Get internal sample data as boxed slice of *u8*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Samp24, COUNT>) -> Self {
        let mut frames: Vec<Frame<Samp24, COUNT>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Samp24, COUNT>>() / 3;
        let buffer: Box<[u8]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut u8;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const COUNT: usize> From<Audio<Samp32, COUNT>> for Box<[f32]> {
    /// Get internal sample data as boxed slice of *f32*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Samp32, COUNT>) -> Self {
        let mut frames: Vec<Frame<Samp32, COUNT>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Samp32, COUNT>>() / 4;
        let buffer: Box<[f32]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<const COUNT: usize> From<Audio<Samp64, COUNT>> for Box<[f64]> {
    /// Get internal sample data as boxed slice of *f64*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<Samp64, COUNT>) -> Self {
        let mut frames: Vec<Frame<Samp64, COUNT>> = audio.frames.into();
        let capacity = frames.len() * size_of::<Frame<Samp64, COUNT>>() / 8;
        let buffer: Box<[f64]> = unsafe {
            let ptr = frames.as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
