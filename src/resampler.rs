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
use crate::Audio;
use crate::Stream;

mod speex;

use speex::ResamplerState;

const WINDOW_FN_KAISER_TABLE: &[f64] = &[
    0.99537781, 1.0, 0.99537781, 0.98162644, 0.95908712, 0.92831446,
    0.89005583, 0.84522401, 0.79486424, 0.74011713, 0.68217934, 0.62226347,
    0.56155915, 0.5011968, 0.44221549, 0.38553619, 0.33194107, 0.28205962,
    0.23636152, 0.19515633, 0.15859932, 0.1267028, 0.09935205, 0.07632451,
    0.05731132, 0.0419398, 0.02979584, 0.0204451, 0.01345224, 0.00839739,
    0.00488951, 0.00257636, 0.00115101, 0.00035515, 0.0, 0.0,
];
const WINDOW_FN_OVERSAMPLE: usize = 32;

/// Resampled stream returned from
/// [`Stream::resample()`](crate::Stream::resample).
#[derive(Debug)]
pub struct Resampler<S: ?Sized, Chan, const CH: usize>
where
    Chan: Channel,
    S: Stream<Chan, CH>,
{
    /// Phantom data of output channel type.
    _phantom: PhantomData<Chan>,
    /// Target sample rate.
    sample_rate: u32,
    /// Simplified ratio of input ÷ output samples.
    ratio: (u32, u32),
    /// Channel data.
    channels: [Resampler32; CH],
    /// Calculated output latency for resampler.
    output_latency: u32,
    /// Calculated input latency for resampler.
    input_latency: u32,
    /// Source stream.
    stream: S,
}

impl<'a, S, Chan, const CH: usize> Resampler<S, Chan, CH>
where
    Chan: Channel,
    S: Stream<Chan, CH>,
    Ch32: From<Chan>,
{
    /// Switch source stream for resampler.
    pub fn switch<Z>(self, stream: Z) -> Resampler<Z, Chan, CH>
    where
        Z: Stream<Chan, CH>,
    {
        // Target sample rate always stays the same.
        let sample_rate = self.sample_rate;
        // Calculate new simplified ratio of input ÷ output samples.
        let ratio = simplify(stream.sample_rate(), sample_rate);
        // Channels may need to change if input sample rate is changed.
        let mut channels = self.channels;
        // Latency will change with different sample rates as well.
        let mut output_latency = self.output_latency;
        let mut input_latency = self.input_latency;

        // Handle sample rate change, if needed.
        if stream.sample_rate() != self.stream.sample_rate() {
            // Prepare each channel for sample rate change
            for ch in channels.iter_mut() {
                let state = &mut ch.state;
                let num = ratio.0;
                let den = ratio.1;

                let v = state.samp_frac_num;
                speex::_muldiv(&mut state.samp_frac_num, v, den, self.ratio.1);
                if state.samp_frac_num >= den {
                    state.samp_frac_num = den - 1;
                }

                state.update_filter(num, den);

                input_latency = state.filt_len / 2;
                output_latency = (input_latency * den + (num >> 1)) / num;
            }
        }

        //
        Resampler {
            _phantom: PhantomData,
            sample_rate,
            ratio,
            channels,
            output_latency,
            input_latency,
            stream,
        }
    }

    /// Create a new resampler.
    pub(crate) fn new(new_hz: u32, stream: S) -> Self {
        // FIXME remove when for impl Default for T on [T; N]
        use std::convert::TryInto;

        // Calculate simplified ratio of input ÷ output samples.
        let ratio = simplify(stream.sample_rate(), new_hz);
        let mut this = Self {
            _phantom: PhantomData,
            sample_rate: new_hz,
            ratio,
            stream,
            channels: vec![Default::default(); CH].try_into().unwrap(),
            output_latency: 0,
            input_latency: 0,
        };
        for channel in this.channels.iter_mut() {
            let num = ratio.0;
            let den = ratio.1;

            channel.state.update_filter(num, den);

            // Get input latency.
            let input_latency = channel.state.filt_len / 2;
            // Get output latency.
            let output_latency = (input_latency * den + (num >> 1)) / num;

            this.output_latency = output_latency;
            this.input_latency = input_latency;
        }

        this
    }
}

impl<'a, S, Chan, const CH: usize> Stream<Chan, CH> for Resampler<S, Chan, CH>
where
    Chan: Channel,
    S: Stream<Chan, CH>,
    Ch32: From<Chan>,
{
    #[inline(always)]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline(always)]
    fn sink<C: Channel, const N: usize>(&mut self, buf: &mut Audio<C, N>)
    where
        C: From<Chan>,
    {
        // Make sure target sample rate is the same.
        assert_eq!(buf.sample_rate(), self.sample_rate);

        let len = buf.len();
        // First, de-interleave input audio data into f32 buffer.
        let input_samples: u32 = self.input_latency
            + (len as u64 * self.ratio.0 as u64 / self.ratio.1 as u64) as u32;
        let mut convert = Audio::<Ch32, CH>::with_silence(
            self.stream.sample_rate(),
            input_samples as usize,
        );
        self.stream.sink(&mut convert);
        for chan in 0..CH {
            self.channels[chan].input.clear();
        }
        for frame in convert.iter() {
            for chan in 0..CH {
                self.channels[chan].input.push(frame.0[chan].to_f32());
            }
        }

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

        // Then, re-interleave the samples back.
        for i in 0..len {
            let mut frame = Frame::<C, CH>::default();
            for chan in 0..CH {
                frame.0[chan] = C::from(self.channels[chan].output[i]);
            }
            *buf.get_mut(i).unwrap() = frame.to();
        }
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
