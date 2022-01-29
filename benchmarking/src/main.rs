use nanorand::{Rng, WyRand};
use std::time::Instant;

// Number of i16 samples (8mb)
const SIZE: usize = 4_194_304;

fn gen_buffer() -> Vec<i16> {
    let mut rng = WyRand::new();
    let mut buffer = Vec::with_capacity(SIZE);
    for _ in 0..SIZE {
        buffer.push(rng.generate::<i16>());
    }
    buffer
}

fn main() {
    ////
    std::thread::sleep(std::time::Duration::from_millis(200));

    let samples = gen_buffer();
    let start = Instant::now();

    let sample_rate = 48_000;
    let audio = fon5::Audio::<fon5::stereo::Stereo16>::with_i16_buffer(
        sample_rate,
        samples.into_boxed_slice(),
    );
    let mut mono32 =
        fon5::Audio::<fon5::mono::Mono32>::with_stream(sample_rate, &audio);
    let slice = mono32.as_f32_slice();
    std::convert::identity(slice);

    let elapsed = start.elapsed();
    println!("fon5 {}µs", elapsed.as_micros());

    ////
    std::thread::sleep(std::time::Duration::from_millis(200));

    let samples = gen_buffer();
    let start = Instant::now();

    let sample_rate = 48_000;
    let audio = fon6::Audio::<fon6::chan::Ch16, 2>::with_i16_buffer(
        sample_rate,
        samples.into_boxed_slice(),
    );
    let mut mono32 =
        fon6::Audio::<fon6::chan::Ch32, 1>::with_audio(sample_rate, &audio);
    let slice = mono32.as_f32_slice();
    std::convert::identity(slice);

    let elapsed = start.elapsed();
    println!("fon6 {}µs", elapsed.as_micros());
}
