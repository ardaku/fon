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
use core::num::NonZeroU32;

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
    denominator: NonZeroU32,
    /// Source stream.
    stream: S,
    /// Input buffer (audio from source stream).
    buffer: Audio<Ch32, CH, SR>,
    /// Output buffer (audio from source stream).
    output: Audio<Ch32, CH, SR>,
    /// State (For all channels)
    // FIXME: Use const generic arrays instead of `Vec`s.
    state: [ResamplerState; CH],
    /* /// De-interleaved f32 input buffers.
    input: [Vec<f32>; CH],
    /// De-interleaved f32 output buffers.
    output: [Vec<f32>; CH], */
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
        use std::convert::TryInto; // wait for impl Default for T on [T; N]

        // Calculate simplified ratio of input ÷ output samples.
        let ratio = simplify(
            NonZeroU32::new(SR).unwrap(),
            NonZeroU32::new(HZ).unwrap(),
        );
        let mut this = Self {
            _phantom: PhantomData,
            denominator: ratio.1,
            stream,
            buffer: Audio::with_silence(0),
            output: Audio::with_silence(0),
            state: vec![Default::default(); CH].try_into().unwrap(),
        };
        for channel in this.state.iter_mut() {
            let num = ratio.0.get();
            let den = ratio.1.get();

            channel.update_filter(num, den);

            // Get output latency.
            let output_latency =
                (((channel.filt_len / 2) * den + (num >> 1)) / num) as usize;
            // Get input latency.
            let input_latency = (channel.filt_len / 2) as usize;

            dbg!(input_latency, output_latency);
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
        // Get the ratio of input to output samples
        let ratio_io = SR as f64 / HZ as f64;
        // Calculate the number of input samples required to fill the output
        let input: usize = (len as f64 * ratio_io).ceil() as usize - 1; // FIXME

        // Set internal audio input buffer to `input` samples from the stream
        let mut convert = Audio::with_silence(0);
        self.stream.extend(&mut convert, input);
        self.buffer = Audio::with_stream(convert, input);

        // Set internal output audio buffer length.
        self.output.0.resize(len, Default::default());

        // Resample interleaved audio data.
        for (i, state) in self.state.iter_mut().enumerate() {
            state.out_stride = CH as u32;
            state.in_stride = CH as u32;

            let mut input = input as u32;
            let mut len2 = len as u32;
            dbg!(len2);
            state.process_float(
                &self.buffer.as_f32_slice()[i..],
                &mut input,
                &mut self.output.as_f32_slice()[i..],
                &mut len2,
                self.denominator.get(),
            );
            dbg!(len2);
            assert_eq!(len2 as usize, len);
        }

        // Write to output buffer.
        buffer.0.extend(self.output.iter().map(|x| x.to()));
    }
}

// Simplify a ratio (fraction with non-zero numerator and denominator).
#[inline(always)]
fn simplify(num: NonZeroU32, den: NonZeroU32) -> (NonZeroU32, NonZeroU32) {
    let factor = gcd(num.get(), den.get());
    (
        NonZeroU32::new(num.get() / factor).unwrap(),
        NonZeroU32::new(den.get() / factor).unwrap(),
    )
}

// Calculate the greatest common divisor of two 32-bit integers.
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
