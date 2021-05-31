use std::convert::TryInto;

use fon::chan::Ch32;
use fon::{Audio, Resampler};

// Resample an audio file from one sample rate to another.
fn resample<const IN_HZ: u32, const OUT_HZ: u32>(
    in_file: &str,
    out_file: &str,
) {
    // Load file as f32 buffer.
    let rawfile = std::fs::read(in_file).unwrap();
    let mut audio = Vec::new();
    for sample in rawfile.chunks(4) {
        audio.push(f32::from_le_bytes(sample.try_into().unwrap()));
    }
    // Create type-safe audio type from f32 buffer.
    let audio = Audio::<Ch32, 2, IN_HZ>::with_f32_buffer(audio);
    // Calculate new length after resampling.
    let len =
        (audio.len() as f64 * OUT_HZ as f64 / IN_HZ as f64).ceil() as usize;
    // Create resampler wrapping audio.
    let resampler = Resampler::new(audio);
    // Stream resampler into new audio type.
    let mut audio = Audio::<Ch32, 2, OUT_HZ>::with_stream(resampler, len);
    // Write file as f32 buffer.
    let mut bytes = Vec::new();
    for sample in audio.as_f32_slice() {
        bytes.extend(&sample.to_le_bytes());
    }
    std::fs::write(out_file, bytes).unwrap();
}

fn main() {
    resample::<44_100, 48_000>("examples/44_1k.raw", "48k.raw");
    resample::<48_000, 44_100>("examples/48k.raw", "44_1k.raw");
}
