use fon::chan::{Ch16, Ch32};
use fon::{Audio, Sink, Stream};

fn main() {
    /*
    // Load stereo 16-bit LE PCM at 44_100 Hz.
    let rawfile = std::fs::read("examples/audio.raw").unwrap();
    // Convert from bytes into a 16-bit PCM array
    let mut audio = Vec::new();
    for sample in rawfile.chunks(2) {
        audio.push(i16::from_le_bytes([sample[0], sample[1]]));
    }
    // Convert from 16-bit PCM array into fon Audio buffer.
    let audio = Audio::<Stereo16>::with_i16_buffer(44_100, audio);
    // Change the sample rate.
    let audio = Audio::<Stereo32>::with_stream(48_000, &audio);
    // Write out a new 32-bit LE Float PCM file at the new sample rate.
    let mut output = Vec::new();
    for sample in audio.as_f32_slice() {
        output.extend(&sample.to_le_bytes());
    }
    std::fs::write("output.raw", output).unwrap();*/

    // Load stereo 16-bit LE PCM at 44_100 Hz.
    let rawfile = std::fs::read("examples/audio.raw").unwrap();
    // Convert from bytes into a 16-bit PCM array
    let mut audio = Vec::new();
    for sample in rawfile.chunks(2) {
        audio.push(i16::from_le_bytes([sample[0], sample[1]]));
    }
    // Convert from 16-bit PCM array into fon Audio buffer.
    let audio = Audio::<Ch16, 2, 44_100>::with_i16_buffer(audio);
    // Change the sample rate.
    let mut output = Audio::<Ch32, 2, 48_000>::with_stream(&audio);

    // Overwrite it by converting it sample by sample.
    {
        let mut sink = output.sink(..);

        for i in 0..audio.len() {
            let mut thing = Stream::take(audio.get(i).unwrap(), 1);
            thing.set_sample_rate(44_100);
            sink.stream(thing);
        }
    }

    // Write out a new 32-bit LE Float PCM file at the new sample rate.
    let mut bytes = Vec::new();
    for sample in output.as_f32_slice() {
        bytes.extend(&sample.to_le_bytes());
    }
    std::fs::write("output.raw", bytes).unwrap();
}
