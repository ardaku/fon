// Copyright © 2020-2021 The Fon Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::mem;

use crate::chan::{Ch32, Channel};
use crate::frame::Frame;
use crate::{Audio, Sink};

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

/// Stream resampler.
#[derive(Debug)]
pub struct Stream<const CH: usize> {
    /// Target sample rate (constant).
    output_sample_rate: u32,
    /// Source sample rate (changeable)
    input_sample_rate: u32,
    /// Simplified ratio of input ÷ output samples.
    ratio: (u32, u32),
    /// Channel data.
    channels: [Resampler32; 8],
    /// Calculated input latency for resampler.
    input_latency: u32,
}

impl<const CH: usize> Stream<CH> {
    /// Create a new stream.
    pub fn new(in_hz: u32, out_hz: u32) -> Self {
        // Calculate simplified ratio of input ÷ output samples.
        let ratio = simplify(in_hz, out_hz);
        let mut this = Self {
            output_sample_rate: out_hz,
            input_sample_rate: in_hz,
            ratio,
            channels: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            input_latency: 0,
        };
        for channel in this.channels.iter_mut() {
            let num = ratio.0;
            let den = ratio.1;

            channel.state.update_filter(num, den);

            // Get input latency.
            this.input_latency = channel.state.filt_len / 2;
        }

        this
    }

    /// Switch source sample rate.
    fn source_hz(&mut self, hz: u32) {
        // Calculate new simplified ratio of input ÷ output samples.
        let ratio = simplify(hz, self.output_sample_rate);

        // Handle sample rate change, if needed.
        if hz != self.input_sample_rate {
            // Prepare each channel for sample rate change
            for ch in self.channels.iter_mut() {
                let state = &mut ch.state;
                let num = ratio.0;
                let den = ratio.1;

                let v = state.samp_frac_num;
                state.samp_frac_num = speex::_muldiv(v, den, self.ratio.1);
                if state.samp_frac_num >= den {
                    state.samp_frac_num = den - 1;
                }

                state.update_filter(num, den);

                self.input_latency = state.filt_len / 2;
            }
            self.ratio = ratio;
        }
    }

    /// Flush audio to sink and end stream.
    pub fn flush<Ch, S>(mut self, sink: S)
    where
        Ch: Channel,
        S: Sink<Ch, CH>,
    {
        // Generate silence.
        for chan in 0..CH {
            self.channels[chan].input.clear();
        }
        for _ in 0..self.input_latency {
            for chan in 0..CH {
                self.channels[chan].input.push(0.0);
            }
        }

        // Resample and output audio to sink.
        let len = sink.len();
        self.resample_audio(
            sink,
            len,
            self.input_latency,
        );
    }

    /// Pipe audio through this stream, and out to the sink.
    ///
    /// If the sink gets full, then no more audio will be written.  If there is
    /// not enough audio then the sink chooses whether or not to fill the rest
    /// of it's buffer with silence.
    pub fn pipe<Chan, Ch, S>(&mut self, audio: &Audio<Chan, CH>, sink: S)
    where
        Chan: Channel,
        Ch: Channel,
        S: Sink<Ch, CH>,
        Ch32: From<Chan>,
    {
        // Make sure that the sample rates match.
        assert_eq!(sink.sample_rate(), self.output_sample_rate);
        if audio.sample_rate() != self.input_sample_rate {
            self.source_hz(audio.sample_rate());
            debug_assert_eq!(audio.sample_rate(), self.input_sample_rate);
        }

        // Get the output length from the sink.
        let len = sink.len();
        // First, de-interleave input audio data into f32 buffer.
        let converted = Audio::<Ch32, CH>::with_frames(
            audio.sample_rate(),
            audio
                .as_slice()
                .iter()
                .map(|frame| frame.to())
                .collect::<Vec<_>>(),
        );
        for chan in 0..CH {
            self.channels[chan].input.clear();
        }
        for frame in converted.iter() {
            for chan in 0..CH {
                self.channels[chan]
                    .input
                    .push(frame.channels()[chan].to_f32());
            }
        }

        // Next, allocate space for output channels and resample.
        self.resample_audio(sink, len, audio.len() as u32);
    }

    fn resample_audio<Ch, S>(
        &mut self,
        mut sink: S,
        len: usize,
        input_samples: u32,
    ) where
        Ch: Channel,
        S: Sink<Ch, CH>,
    {
        let mut out = 0;

        // Allocate space for output channels and resample
        for chan in 0..CH {
            self.channels[chan].output.resize(len, 0.0);

            let mut in_ = input_samples;
            out = len as u32;

            debug_assert_eq!(in_, self.channels[chan].input.len() as u32);
            debug_assert_eq!(out, self.channels[chan].output.len() as u32);

            self.channels[chan].state.process_float(
                self.channels[chan].input.as_slice(),
                &mut in_,
                self.channels[chan].output.as_mut_slice(),
                &mut out,
                self.ratio.1,
            );
        }

        // Then, re-interleave the samples back.
        sink.sink_with(&mut (0..out as usize).into_iter().map(|i| {
            let mut out_frame = Frame::<Ch, CH>::default();
            for chan in 0..CH {
                out_frame.channels_mut()[chan] =
                    Ch::from(self.channels[chan].output[i]);
            }
            out_frame
        }));
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
