use alloc::vec::Vec;
use core::{array, mem, num::NonZeroU32};

use crate::{
    chan::{Channel, Samp32},
    frame::Frame,
    Audio, Sink,
};

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

/// Resampler stream
#[derive(Debug)]
pub struct Resampler<const N: usize> {
    /// Target sample rate (constant).
    output_sample_rate: u32,
    /// Source sample rate (changeable)
    input_sample_rate: Option<NonZeroU32>,
    /// Simplified ratio of input ÷ output samples.
    ratio: (u32, u32),
    /// Sample data.
    samples: [Resampler32; N],
    /// Calculated input latency for resampler.
    input_latency: u32,
}

impl<const N: usize> Resampler<N> {
    /// Create a new stream at target sample rate.
    pub fn new(target_hz: u32) -> Self {
        assert_ne!(target_hz, 0);
        Self {
            output_sample_rate: target_hz,
            input_sample_rate: None,
            ratio: (0, 1),
            samples: array::from_fn(|_| Default::default()),
            input_latency: 0,
        }
    }

    /// Switch source sample rate.
    fn source_hz(&mut self, hz: NonZeroU32) {
        // Calculate new simplified ratio of input ÷ output samples.
        let ratio = simplify(hz.get(), self.output_sample_rate);
        let (num, den) = ratio;

        // Handle sample rate change, if needed.
        if NonZeroU32::new(hz.get()) != self.input_sample_rate {
            // Prepare each channel for sample rate change
            for ch in self.samples.iter_mut() {
                // Store fractional sample data.
                let v = ch.state.samp_frac_num;
                ch.state.samp_frac_num = speex::_muldiv(v, den, self.ratio.1);
                if ch.state.samp_frac_num >= den {
                    ch.state.samp_frac_num = den - 1;
                }

                // Update filter and calculate input latency.
                ch.state.update_filter(num, den);
                self.input_latency = ch.state.filt_len / 2;
            }
            self.ratio = ratio;
            self.input_sample_rate = Some(hz);
        }
    }

    /// Flush audio to sink and end stream.
    pub fn flush<C, K>(mut self, sink: K)
    where
        C: Channel,
        K: Sink<C, N>,
    {
        if self.samples[0].state.started == 0 {
            return;
        }

        // Generate silence.
        for samp in 0..N {
            self.samples[samp].input.clear();
        }
        for _ in 0..self.input_latency {
            for samp in 0..N {
                self.samples[samp].input.push(0.0);
            }
        }

        // Resample and output audio to sink.
        self.resample_audio(sink);
    }

    /// Pipe audio through this stream, and out to the sink.
    ///
    /// If the sink gets full, then no more audio will be written.  If there is
    /// not enough audio then the sink chooses whether or not to fill the rest
    /// of it's buffer with silence.
    pub fn pipe<F, C, K>(&mut self, audio: &Audio<F, N>, mut sink: K)
    where
        F: Channel,
        C: Channel + From<F>,
        K: Sink<C, N>,
        Samp32: From<F>,
    {
        // Make sure that the sample rates match.
        assert_eq!(sink.sample_rate().get(), self.output_sample_rate);

        // If sample rates match, do a copy (faster than resampling).
        if self.samples[0].state.started == 0
            && sink.sample_rate() == audio.sample_rate()
        {
            sink.sink_with(&mut audio.iter().cloned().map(|x| x.to()));
            return;
        }

        // Change source sample rate if it doesn't match.
        if NonZeroU32::new(audio.sample_rate().get()) != self.input_sample_rate
        {
            self.source_hz(audio.sample_rate());
        }

        // First, de-interleave input audio data into f32 buffer.
        let converted = Audio::<Samp32, N>::with_frames(
            audio.sample_rate().get(),
            audio
                .as_slice()
                .iter()
                .map(|frame| frame.to())
                .collect::<Vec<_>>(),
        );
        for samp in 0..N {
            self.samples[samp].input.clear();
        }
        for frame in converted.iter() {
            for samp in 0..N {
                self.samples[samp]
                    .input
                    .push(frame.samples()[samp].to_f32());
            }
        }

        // Next, allocate space for output samples and resample.
        self.resample_audio(sink);
    }

    fn resample_audio<C, K>(&mut self, mut sink: K)
    where
        C: Channel,
        K: Sink<C, N>,
    {
        // If no input samples, skip doing the work.
        if self.samples[0].input.is_empty() {
            return;
        }

        let mut out = u32::MAX;

        // Allocate space for output samples and resample
        for samp in 0..N {
            self.samples[samp].output.resize(sink.len(), 0.0);

            // FIXME: Remove length parameters, return number of output samples.
            self.samples[samp].state.process_float(
                self.samples[samp].input.as_slice(),
                &mut (self.samples[samp].input.len() as u32),
                self.samples[samp].output.as_mut_slice(),
                &mut out,
                self.ratio.1,
            );
        }

        // Then, re-interleave the samples back.
        sink.sink_with(&mut (0..out as usize).map(|i| {
            let mut out_frame = Frame::<C, N>::default();
            for samp in 0..N {
                out_frame.samples_mut()[samp] =
                    C::from(self.samples[samp].output[i]);
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
