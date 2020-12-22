// Copyright Jeron Aldaron Lau 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Rust audio types and conversions.
//!
//! An [audio buffer] can be cheaply converted to and from raw samples (i8, i16,
//! f32, and f64) buffers, enabling interoperability with other crates.
//!
//! Many audio formats are supported:
//! - Any sample rate
//! - Bit depth: [8]- or [16]-bit integer and [32]- or [64]-bit float
//! - [Mono], [Stereo], [5.1 Surround]
//!
//! Blending [operations] are supported for all formats.
//!
//! # 8-Bit Sawtooth Wave Example
//! ```rust
//! use fon::chan::Ch8;
//! use fon::mono::Mono8;
//! use fon::stereo::Stereo16;
//! use fon::{Audio, Frame};
//!
//! let mut a = Audio::<Mono8>::with_silence(44_100, 256);
//! for (i, s) in a.iter_mut().enumerate() {
//!     s.channels_mut()[0] = Ch8::new(i as i8);
//! }
//! // Convert to stereo 16-Bit 48_000 KHz audio format
//! let audio = Audio::<Stereo16>::with_audio(48_000, &a);
//! ```
//!
//! [audio buffer]: crate::Audio
//! [8]: crate::chan::Ch8
//! [16]: crate::chan::Ch16
//! [32]: crate::chan::Ch32
//! [64]: crate::chan::Ch64
//! [Mono]: crate::mono::Mono
//! [Stereo]: crate::stereo::Stereo
//! [5.1 Surround]: crate::surround::Surround
//! [operations]: crate::ops

mod audio;
pub mod chan;
mod frame;
pub mod mono;
pub mod ops;
mod private;
pub mod stereo;
mod streaming;
pub mod surround;

pub use audio::Audio;
pub use frame::Frame;
pub use streaming::{Resampler, Sink, Stream};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
