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
    ///
    _phantom: PhantomData<Chan>,
    /// Source stream.
    stream: S,
    /// Input buffer (audio from source stream).
    buffer: Audio<Ch32, CH, SR>,
    /// Output buffer (audio from source stream).
    output: Audio<Ch32, CH, SR>,
    /// State (For all channels)
    // FIXME: Use const generic arrays instead of `Vec`s.
    state: [ResamplerState; CH],
}

use std::convert::TryInto;
macro_rules! rep {
    ($a:expr; $b:expr) => {
        vec![$a; $b].try_into().unwrap()
    };
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
        Self {
            _phantom: PhantomData,
            stream,
            buffer: Audio::with_silence(0),
            output: Audio::with_silence(0),
            state: rep![ResamplerState::new(CH /*fixme: should be const generic */,
                SR /*fixme: input hz should be const generic */ ,
                HZ /*fixme: output hz should be const generic */ ); CH],
        }
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
        let input: usize = (len as f64 * ratio_io).floor() as usize - 1; // FIXME

        // Set internal audio input buffer to `input` samples from the stream
        let mut convert = Audio::with_silence(0);
        self.stream.extend(&mut convert, input);
        self.buffer = Audio::with_stream(convert, input);

        // Set internal output audio buffer length.
        self.output.0.resize(len, Default::default());

        // Resample interleaved audio data.
        for (i, state) in self.state.iter_mut().enumerate() {
            dbg!(
                state.get_input_latency(),
                state.get_output_latency(),
                state.get_ratio()
            );

            state.out_stride = CH as u32;
            state.in_stride = CH as u32;

            let mut input = input as u32;
            let mut len2 = len as u32;
            dbg!(len2);
            state.process_float(
                0,
                &self.buffer.as_f32_slice()[i..],
                &mut input,
                &mut self.output.as_f32_slice()[i..],
                &mut len2,
            );
            dbg!(len2);
            assert_eq!(len2 as usize, len);
        }

        // Write to output buffer.
        buffer.0.extend(self.output.iter().map(|x| x.to()));
    }
}
