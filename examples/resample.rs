use std::convert::TryInto;

use fon::chan::Ch32;
use fon::{Audio, Stream};

// Resample an audio file from one sample rate to another.
fn resample(in_hz: u32, in_file: &str, out_hz: u32, out_file: &str) {
    // Load file as f32 buffer.
    let rawfile = std::fs::read(in_file).unwrap();
    let mut audio = Vec::new();
    for sample in rawfile.chunks(4) {
        audio.push(f32::from_le_bytes(sample.try_into().unwrap()));
    }
    // Create type-safe audio type from f32 buffer.
    let audio = Audio::<Ch32, 2>::with_f32_buffer(in_hz, audio);
    // Calculate new length after resampling.
    let len =
        (audio.len() as f64 * out_hz as f64 / in_hz as f64).ceil() as usize;
    // Create resampler wrapping audio.
    let resampler = audio.resample(out_hz);
    // Stream resampler into new audio type.
    let mut audio = Audio::<Ch32, 2>::with_stream(resampler, len);
    // Write file as f32 buffer.
    let mut bytes = Vec::new();
    for sample in audio.as_f32_slice() {
        bytes.extend(&sample.to_le_bytes());
    }
    std::fs::write(out_file, bytes).unwrap();
}

fn main() {
    resample(44_100, "examples/44_1k.raw", 48_000, "48k.raw");
    resample(48_000, "examples/48k.raw", 44_100, "44_1k.raw");
}
