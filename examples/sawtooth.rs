use fon::chan::Ch8;
use fon::mono::{Mono16, Mono8};
use fon::stereo::Stereo16;
use fon::{Audio, Sample};

fn main() {
    let mut a = Audio::<Mono8>::with_silence(44_100, 256);
    for (i, s) in a.iter_mut().enumerate() {
        s.channels_mut()[0] = Ch8::new((i as i16 - 128) as i8);
    }
    // Convert to stereo 16-Bit 48_000 KHz audio format
    let audio = Audio::<Mono8>::with_audio(48_000, &a);

    // Print out original wave.
    /*for sample in a.as_i8_slice() {
        println!("{}", sample);
    }*/

    // Print out converted wave.
    for sample in audio.as_i8_slice() {
        println!("{}", sample);
    }
}
