// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::marker::PhantomData;
use core::mem;

use crate::chan::{Ch32, Channel};
use crate::frame::Frame;
use crate::ops::Ops;
use crate::Audio;
use crate::Stream;

mod speex;

use speex::ResamplerState;

/// Resampler stream.  Wraps a stream, and implements `Stream` with a different
/// sample rate.
#[derive(Debug)]
pub struct Resampler<S, Chan, const CH: usize, const SR: u32, const HZ: u32>
where
    Chan: Channel,
    S: Stream<Chan, CH, SR>,
    Frame<Chan, CH>: Ops<Chan>,
    Frame<Ch32, CH>: Ops<Ch32>,
{
    /// Phantom data of output channel type.
    _phantom: PhantomData<Chan>,
    /// Denominator of the simplified ratio of input ÷ output samples.
    ratio: (u32, u32),
    /// Source stream.
    stream: S,
    /// Input buffer (audio from source stream).
    buffer: Audio<Ch32, CH, SR>,
    /// Output buffer (audio from source stream).
    output: Audio<Ch32, CH, SR>,
    /// Channel data.
    channels: [Resampler32; CH],
    /// Calculated output latency for resampler.
    output_latency: u32,
    /// Calculated input latency for resampler.
    input_latency: u32,
}

impl<'a, S, Chan, const CH: usize, const SR: u32, const HZ: u32>
    Resampler<S, Chan, CH, SR, HZ>
where
    Chan: Channel,
    S: Stream<Chan, CH, SR>,
    Frame<Chan, CH>: Ops<Chan>,
    Frame<Ch32, CH>: Ops<Ch32>,
    Ch32: From<Chan>,
{
    /// Create a new resampler.
    pub fn new(stream: S) -> Self {
        // FIXME remove when for impl Default for T on [T; N]
        use std::convert::TryInto;

        // Calculate simplified ratio of input ÷ output samples.
        let ratio = simplify(SR, HZ);
        let mut this = Self {
            _phantom: PhantomData,
            ratio,
            stream,
            buffer: Audio::with_silence(0),
            output: Audio::with_silence(0),
            channels: vec![Default::default(); CH].try_into().unwrap(),
            output_latency: 0,
            input_latency: 0,
        };
        for channel in this.channels.iter_mut() {
            let num = ratio.0;
            let den = ratio.1;

            channel.state.update_filter(num, den);

            // Get input latency.
            let input_latency = (channel.state.filt_len / 2);
            // Get output latency.
            let output_latency = (input_latency * den + (num >> 1)) / num;

            dbg!(input_latency, output_latency);
            
            this.output_latency = output_latency;
            this.input_latency = input_latency;
        }

        this
    }
}

impl<'a, S, Chan, const CH: usize, const SR: u32, const HZ: u32>
    Stream<Chan, CH, HZ> for Resampler<S, Chan, CH, SR, HZ>
where
    Chan: Channel,
    S: Stream<Chan, CH, SR>,
    Frame<Chan, CH>: Ops<Chan>,
    Frame<Ch32, CH>: Ops<Ch32>,
    Ch32: From<Chan>,
{
    #[inline(always)]
    fn extend<C: Channel>(&mut self, buffer: &mut Audio<C, CH, HZ>, len: usize)
    where
        C: From<Chan>,
        Frame<C, CH>: Ops<C>,
    {
        // First, de-interleave input audio data into f32 buffer.
        let len_plus_latency = len as u64;
        let input_samples: u32 = self.input_latency +
            (len_plus_latency * self.ratio.0 as u64 / self.ratio.1 as u64) as u32;
        let mut convert = Audio::<Ch32, CH, SR>::with_silence(0);
        self.stream.extend(&mut convert, input_samples as usize);
        for frame in convert.iter() {
            for chan in 0..CH {
                self.channels[chan].input.push(frame.0[chan].to_f32());
            }
        }
        println!("De-interleaved!");

        // Next, allocate space for output channels and resample.
        for chan in 0..CH {
            self.channels[chan].output.resize(len, 0.0);

            let mut in_ = input_samples;
            let mut out = len as u32;

            assert_eq!(in_, self.channels[chan].input.len() as u32);
            assert_eq!(out, self.channels[chan].output.len() as u32);

            self.channels[chan].state.process_float(
                self.channels[chan].input.as_slice(),
                &mut in_,
                self.channels[chan].output.as_mut_slice(),
                &mut out,
                self.ratio.1,
            );

            assert_eq!(out, len as u32);
            assert_eq!(in_, input_samples);
        }
        println!("Resampled!");

        // Then, re-interleave the samples back.
        buffer.0.reserve(len);
        for i in 0..len {
            let mut frame = Frame::<C, CH>::default();
            for chan in 0..CH {
                frame.0[chan] = C::from(self.channels[chan].output[i]);
            }
            buffer.0.push_back(frame);
        }
        println!("Re-interleaved!");

        /*let input_samples: (u32, u32) = (self.ratio.0 * len_plus_latency, self.ratio.1);
        let input_samples = input_samples.0 / input_samples.1
            + (input_samples.0 % input_samples.1 != 0) as u32;*/
        // dbg!(input_samples);
    
        /*// Get the ratio of input to output samples
        let ratio_io = SR as f64 / HZ as f64;
        // Calculate the number of input samples required to fill the output
        let input: usize = (len as f64 * ratio_io).ceil() as usize - 1; // FIXME

        dbg!(input);

        // Set internal audio input buffer to `input` samples from the stream
        let mut convert = Audio::with_silence(0);
        self.stream.extend(&mut convert, input);
        self.buffer = Audio::with_stream(convert, input);

        // Set internal output audio buffer length.
        self.output.0.resize(len, Default::default());

        // Resample interleaved audio data.
        for (i, ch) in self.channels.iter_mut().enumerate() {
            ch.state.out_stride = CH as u32;
            ch.state.in_stride = CH as u32;

            let mut input = input as u32;
            let mut len2 = len as u32;
            dbg!(len2);
            ch.state.process_float(
                &self.buffer.as_f32_slice()[i..],
                &mut input,
                &mut self.output.as_f32_slice()[i..],
                &mut len2,
                self.ratio.1,
            );
            dbg!(len2);
            assert_eq!(len2 as usize, len);
        }

        // Write to output buffer.
        buffer.0.extend(self.output.iter().map(|x| x.to()));*/
    }
}

/// Single-channel resampler data.
#[derive(Default, Clone, Debug)]
struct Resampler32 {
    // FIXME: Remove state.
    state: ResamplerState,
    // De-interleaved input audio stream for a single channel.
    input: Vec<f32>,
    // De-interleaved output audio stream for a single channel.
    output: Vec<f32>,
}

/// Simplify a ratio (fraction with non-zero numerator and denominator).
#[inline(always)]
fn simplify(num: u32, den: u32) -> (u32, u32) {
    debug_assert_ne!(num, 0);
    debug_assert_ne!(den, 0);

    let factor = gcd(num, den);
    (num / factor, den / factor)
}

/// Calculate the greatest common divisor of two 32-bit integers.
#[inline(always)]
fn gcd(mut a: u32, mut b: u32) -> u32 {
    if b == 0 {
        return a;
    }
    while a != 0 {
        mem::swap(&mut a, &mut b);
        a %= b;
    }
    b
}
