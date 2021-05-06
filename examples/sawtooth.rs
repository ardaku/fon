use fon::chan::{Ch16, Ch32};
use fon::sample::Mono;
use fon::Audio;

fn main() {
    let mut a = Audio::<Ch32, 1>::with_silence(44_100, 256);
    let mut counter = 0.0;
    for f in a.iter_mut() {
        f[Mono] = counter.into();
        counter += 0.05;
    }

    // Convert to stereo 16-Bit 48_000 KHz audio format
    let mut audio = Audio::<Ch16, 2>::with_stream(48_000, &a);

    // Print out converted wave.
    for sample in audio.as_i16_slice() {
        println!("{}", sample);
    }
}
