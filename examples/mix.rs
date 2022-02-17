// Audio mixing example

use fon::{
    chan::{Ch32, Channel},
    Audio, Frame, Sink, Stream,
};
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct Mixer<'a, Chan: Channel, const CH: usize> {
    index: usize,
    audio: &'a mut Audio<Chan, CH>,
}

#[allow(single_use_lifetimes)]
impl<'a, Chan: Channel, const CH: usize> Mixer<'a, Chan, CH> {
    #[inline(always)]
    fn new(audio: &'a mut Audio<Chan, CH>) -> Self {
        let index = 0;

        Mixer { index, audio }
    }
}

// Using '_ results in reserved lifetime error.
#[allow(single_use_lifetimes)]
impl<'a, Chan: Channel, const CH: usize> Sink<Chan, CH>
    for Mixer<'a, Chan, CH>
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
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<Chan, CH>>) {
        let mut this = self;
        Sink::<Chan, CH>::sink_with(&mut this, iter)
    }
}

impl<Chan: Channel, const CH: usize> Sink<Chan, CH>
    for &mut Mixer<'_, Chan, CH>
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
    fn sink_with(&mut self, iter: &mut dyn Iterator<Item = Frame<Chan, CH>>) {
        for frame in self.audio.iter_mut().skip(self.index) {
            if let Some(other) = iter.next() {
                for (channel, chan) in
                    frame.channels_mut().iter_mut().zip(other.channels())
                {
                    *channel += *chan;
                }
            } else {
                break;
            }
            self.index += 1;
        }
    }
}

fn load_file(in_hz: u32, in_file: &str) -> Audio<Ch32, 2> {
    // Load file as f32 buffer.
    let rawfile = std::fs::read(in_file).unwrap();
    let mut audio = Vec::new();
    for sample in rawfile.chunks(4) {
        audio.push(f32::from_le_bytes(sample.try_into().unwrap()));
    }
    // Create type-safe audio type from f32 buffer.
    Audio::with_f32_buffer(in_hz, audio)
}

fn save_file(name: &str, audio: &Audio<Ch32, 2>) -> std::io::Result<()> {
    // Convert audio to byte buffer
    let mut samples = Vec::<u8>::new();
    for frame in audio.iter() {
        for channel in frame.channels() {
            samples.extend(channel.to_f32().to_le_bytes());
        }
    }
    // Save byte buffer
    std::fs::write(name, samples)
}

fn main() -> std::io::Result<()> {
    // We are mixing file 1 audio down into file 2 audio.

    // Load file 1
    let source = load_file(44_100, "examples/44_1k.raw");
    // Load file 2
    let mut out = load_file(48_000, "examples/48k.raw");
    // Create mixer sink over output buffer
    let mut mixer = Mixer::new(&mut out);

    // Create a stream to convert to 48k
    let mut stream = Stream::<2>::new(48_000);
    stream.pipe(&source, &mut mixer);
    stream.flush(&mut mixer);

    // Save the mixed audio
    save_file("examples/output.raw", &out)
}
