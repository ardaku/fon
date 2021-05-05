use fon::chan::{Ch16, Ch32};
use fon::mono::Mono32;
use fon::stereo::Stereo16;
use fon::{Audio, Frame};

fn main() {
    let mut a = Audio::<Ch32, 1>::with_silence(44_100, 256);
    let mut counter = 0.0;
    for s in a.iter_mut() {
        s.channels_mut()[0] = Ch32::new(counter);
        counter += 0.05;
    }

    // Convert to stereo 16-Bit 48_000 KHz audio format
    let mut audio = Audio::<Ch16, 2>::with_stream(48_000, &a);

    // Print out converted wave.
    for sample in audio.as_i16_slice() {
        println!("{}", sample);
    }
}
