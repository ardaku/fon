use fon::chan::{Ch16, Ch32};
use fon::pos::Mono;
use fon::Audio;

fn main() {
    // Create mono 32-bit floating point audio buffer.
    let mut a = Audio::<Ch32, 1, 48_000>::with_silence(256);
    let mut counter = 0.0;
    for f in a.iter_mut() {
        f[Mono] = counter.into();
        counter += 0.05;
        counter %= 1.0;
    }

    // Convert to stereo 16-Bit audio format
    let mut audio = Audio::<Ch16, 2, 48_000>::with_stream(&a, a.len());

    // Print out converted wave.
    for sample in audio.as_i16_slice() {
        println!("{}", sample);
    }
}
