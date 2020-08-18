// Copyright (c) 2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, or the ZLib
// license <LICENSE-ZLIB or https://www.zlib.net/zlib_license.html> at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    chan::{Ch16, Ch8, Ch32, Ch64},
    ops::Blend,
    sample::Sample,
};
use core::{fmt::Debug, slice::from_raw_parts_mut};

// Channel Identification
// 0. Front Left (Mono)
// 1. Front Right
// 2. Center
// 3. Rear Left
// 4. Rear Right
// 5. LFE
// 6. Side Left
// 7. Side Right

/// Newtype for hertz.
#[derive(Copy, Clone, Debug)]
pub struct Hz(pub f64);

impl From<f64> for Hz {
    fn from(hz: f64) -> Hz {
        Hz(hz)
    }
}

/// An audio buffer (array of audio Samples at a specific sample rate in hertz).
#[derive(Debug)]
pub struct Audio<S: Sample> {
    s_rate: u32,
    samples: Box<[S]>,
}

impl<S: Sample> Audio<S> {
    /// Get view of samples as a slice.
    pub fn as_slice(&self) -> &[S] {
        &self.samples
    }
    
    /// Get view of samples as a mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [S] {
        &mut self.samples
    }

    /// Returns an iterator over the samples.
    pub fn iter(&self) -> std::slice::Iter<'_, S> {
        self.as_slice().iter()
    }
    
    /// Returns an iterator that allows modifying each sample.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, S> {
        self.as_slice_mut().iter_mut()
    }

    /// Construct an `Audio` buffer with all samples set to one value.
    pub fn with_sample(s_rate: u32, len: usize, sample: S) -> Self {
        let samples = vec![sample; len].into_boxed_slice();
        Audio { s_rate, samples }
    }

    /// Construct an `Audio` buffer with all all samples set to the default
    /// value.
    pub fn with_silence(s_rate: u32, len: usize) -> Self {
        Self::with_sample(s_rate, len, S::default())
    }

    /// Construct an `Audio` buffer with another `Audio` buffer.
    ///
    /// The audio format can be converted with this function.
    pub fn with_audio<SrcS: Sample>(s_rate: u32, src: &Audio<SrcS>) -> Self
    where
        S::Chan: From<SrcS::Chan>,
    {
        let src_sr = src.sample_rate();
        if src_sr == s_rate {
            let mut dst = Audio::with_silence(src_sr, src.len());
            // No Resampling Necessary
            for (dst, src) in dst.samples.iter_mut().zip(src.samples.iter()) {
                *dst = src.convert();
            }

            dst
        } else {
            // Resampling Necessary
            let sr_rat = s_rate as f64 / src_sr as f64; // Length ratio
            let dstlen = (sr_rat * src.len() as f64) as usize;
            let mut dst = Audio::with_silence(s_rate, dstlen);

            for (i, dst) in dst.samples.iter_mut().enumerate() {
                let i = sr_rat * i as f64;
                let j = i.trunc() as usize;
                let k = (j + 1).max(src.len() - 1);
                let f = SrcS::from_channels(&[SrcS::Chan::from(i.fract())]);
                *dst = (src.samples[j].lerp(src.samples[k], f)).convert();
            }

            dst
        }
    }

    /// Construct an `Audio` buffer with owned sample data.   You can get
    /// ownership of the pixel data back from the `Audio` buffer as either a
    /// `Vec<S>` or a `Box<[S]>` by calling into().
    pub fn with_samples<B: Into<Box<[S]>>>(s_rate: u32, samples: B) -> Self {
        let samples = samples.into();
        Audio { s_rate, samples }
    }

    /// Construct an `Audio` buffer from an `i8` buffer.
    #[allow(unsafe_code)]
    pub fn with_i8_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i8]>>,
        S: Sample<Chan = Ch8>,
    {
        let buffer: Box<[i8]> = buffer.into();
        let len = buffer.len() / std::mem::size_of::<S>();
        assert_eq!(0, buffer.len() % std::mem::size_of::<S>());
        let slice = Box::<[i8]>::into_raw(buffer);
        let samples: Box<[S]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut S;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, samples }
    }

    /// Construct an `Audio` buffer from an `i16` buffer.
    #[allow(unsafe_code)]
    pub fn with_i16_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[i16]>>,
        S: Sample<Chan = Ch16>,
    {
        let buffer: Box<[i16]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<i16>();
        let len = bytes / std::mem::size_of::<S>();
        assert_eq!(0, bytes % std::mem::size_of::<S>());
        let slice = Box::<[i16]>::into_raw(buffer);
        let samples: Box<[S]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut S;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, samples }
    }
    
    /// Construct an `Audio` buffer from an `f32` buffer.
    #[allow(unsafe_code)]
    pub fn with_f32_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f32]>>,
        S: Sample<Chan = Ch32>,
    {
        let buffer: Box<[f32]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<f32>();
        let len = bytes / std::mem::size_of::<S>();
        assert_eq!(0, bytes % std::mem::size_of::<S>());
        let slice = Box::<[f32]>::into_raw(buffer);
        let samples: Box<[S]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut S;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, samples }
    }
    
    /// Construct an `Audio` buffer from an `f64` buffer.
    #[allow(unsafe_code)]
    pub fn with_f64_buffer<B>(s_rate: u32, buffer: B) -> Self
    where
        B: Into<Box<[f64]>>,
        S: Sample<Chan = Ch64>,
    {
        let buffer: Box<[f64]> = buffer.into();
        let bytes = buffer.len() * std::mem::size_of::<f64>();
        let len = bytes / std::mem::size_of::<S>();
        assert_eq!(0, bytes % std::mem::size_of::<S>());
        let slice = Box::<[f64]>::into_raw(buffer);
        let samples: Box<[S]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut S;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Audio { s_rate, samples }
    }

    /// Get the length of the `Audio` buffer.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if `Audio` buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the sample rate of the `Audio` buffer.
    pub fn sample_rate(&self) -> u32 {
        self.s_rate
    }

    /// Blend `Audio` buffer with a single sample.
    pub fn blend_sample<O: Blend>(&mut self, sample: S, op: O) {
        S::blend_sample(&mut self.samples, &sample, op)
    }

    /// Blend `Audio` buffer with another `Audio` buffer.
    pub fn blend_audio<O: Blend>(&mut self, other: &Self, op: O) {
        S::blend_slice(&mut self.samples, &other.samples, op)
    }
}

impl<S: Sample> From<Audio<S>> for Box<[S]> {
    /// Get internal pixel data as boxed slice.
    fn from(audio: Audio<S>) -> Self {
        audio.samples
    }
}

impl<S: Sample> From<Audio<S>> for Vec<S> {
    /// Get internal pixel data as `Vec` of samples.
    fn from(audio: Audio<S>) -> Self {
        audio.samples.into()
    }
}

impl<S> From<Audio<S>> for Box<[i8]>
where
    S: Sample<Chan = Ch8>,
{
    /// Get internal pixel data as boxed slice of *i8*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<S>) -> Self {
        let samples = audio.samples;
        let capacity = samples.len() * std::mem::size_of::<S>();
        let slice = Box::<[S]>::into_raw(samples);
        let buffer: Box<[i8]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut i8;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<S> From<Audio<S>> for Box<[i16]>
where
    S: Sample<Chan = Ch16>,
{
    /// Get internal pixel data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<S>) -> Self {
        let samples = audio.samples;
        let capacity = samples.len() * std::mem::size_of::<S>() / 2;
        let slice = Box::<[S]>::into_raw(samples);
        let buffer: Box<[i16]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut i16;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}

impl<S> From<Audio<S>> for Box<[f32]>
where
    S: Sample<Chan = Ch32>,
{
    /// Get internal pixel data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<S>) -> Self {
        let samples = audio.samples;
        let capacity = samples.len() * std::mem::size_of::<S>() / 4;
        let slice = Box::<[S]>::into_raw(samples);
        let buffer: Box<[f32]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut f32;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}


impl<S> From<Audio<S>> for Box<[f64]>
where
    S: Sample<Chan = Ch64>,
{
    /// Get internal pixel data as boxed slice of *u16*.
    #[allow(unsafe_code)]
    fn from(audio: Audio<S>) -> Self {
        let samples = audio.samples;
        let capacity = samples.len() * std::mem::size_of::<S>() / 8;
        let slice = Box::<[S]>::into_raw(samples);
        let buffer: Box<[f64]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut f64;
            Box::from_raw(from_raw_parts_mut(ptr, capacity))
        };
        buffer
    }
}
